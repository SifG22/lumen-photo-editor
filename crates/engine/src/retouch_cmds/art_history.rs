//! Art History Brush (`paint.artHistoryBrush`): stylised strokes painted from a history state.
//!
//! Like the History Brush, the colour comes from the targeted surface as it was in a history
//! state (default: the oldest held, the Open snapshot). Instead of copying that state under the
//! brush, every dab scatters short strokes over the `area` around it. Each one takes the source
//! colour where it starts and runs along the source's contours (perpendicular to its brightness
//! gradient), so the result reads as brushwork that follows the image. `style` sets the strokes'
//! length, how far they spread and whether they curl, after Photoshop's ten styles; `tolerance`
//! keeps strokes to where the source differs from the current pixels (0 % paints everywhere).
//!
//! Clean-room: written from Photoshop's documented behaviour (options and their meaning), not its
//! implementation. Deterministic: the same params give the same pixels (`seed` changes them).

use photocraft_color::PixelFormat;
use photocraft_geom::Rect;
use photocraft_paint::retouch::{Region, alpha_index, apply_coverage, stroke_coverage};
use photocraft_paint::{Stroke, StrokePoint, dabs};
use photocraft_raster::Surface;
use serde_json::{Value, json};

use super::{bad, blend_param, damage_json, history_source, num, parse_brush, run_stroke};
use crate::{Result, Session};

const CMD: &str = "paint.artHistoryBrush";

/// Most strokes one command paints, whatever the brush, area and stroke length.
const MAX_STROKES: usize = 4000;
/// Most strokes scattered around one dab.
const MAX_PER_DAB: usize = 32;
/// Most coverage pixels one command renders in all (bounds every stroke's cost on huge brushes).
const PIXEL_BUDGET: u64 = 250_000_000;

/// Photoshop's Art History Brush styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Style {
    TightShort,
    TightMedium,
    TightLong,
    LooseMedium,
    LooseLong,
    Dab,
    TightCurl,
    LooseCurl,
    TightCurlLong,
    LooseCurlLong,
}

impl Style {
    pub(crate) const ALL: [(Style, &'static str); 10] = [
        (Style::TightShort, "tightShort"),
        (Style::TightMedium, "tightMedium"),
        (Style::TightLong, "tightLong"),
        (Style::LooseMedium, "looseMedium"),
        (Style::LooseLong, "looseLong"),
        (Style::Dab, "dab"),
        (Style::TightCurl, "tightCurl"),
        (Style::LooseCurl, "looseCurl"),
        (Style::TightCurlLong, "tightCurlLong"),
        (Style::LooseCurlLong, "looseCurlLong"),
    ];

    fn parse(p: &Value) -> Result<Style> {
        let Some(v) = p.get("style") else { return Ok(Style::TightShort) };
        let name = v.as_str().ok_or_else(|| bad(CMD, "`style` must be a string"))?;
        Style::ALL.iter().find(|(_, n)| *n == name).map(|(s, _)| *s).ok_or_else(|| {
            let names: Vec<&str> = Style::ALL.iter().map(|(_, n)| *n).collect();
            bad(CMD, format!("unknown style `{name}` (one of {})", names.join(", ")))
        })
    }

    fn tight(self) -> bool {
        matches!(self, Style::TightShort | Style::TightMedium | Style::TightLong | Style::TightCurl | Style::TightCurlLong)
    }

    /// Stroke length in brush diameters.
    fn length(self) -> f32 {
        match self {
            Style::Dab => 0.0,
            Style::TightShort => 1.5,
            Style::TightMedium | Style::LooseMedium | Style::TightCurl | Style::LooseCurl => 3.0,
            Style::TightLong | Style::LooseLong | Style::TightCurlLong | Style::LooseCurlLong => 6.0,
        }
    }

    /// Turn per step along the stroke, in radians (0: the stroke follows the contours only).
    fn curl(self) -> f32 {
        match self {
            Style::TightCurl | Style::TightCurlLong => 0.45,
            Style::LooseCurl | Style::LooseCurlLong => 0.25,
            _ => 0.0,
        }
    }

    /// How far from the dab strokes start, as a fraction of the area's radius.
    fn spread(self) -> f32 {
        if self.tight() { 0.6 } else { 1.0 }
    }

    /// Random turn added to each stroke's direction, in radians.
    fn jitter(self) -> f32 {
        match self {
            Style::Dab => std::f32::consts::PI,
            s if s.tight() => 0.15,
            _ => 0.6,
        }
    }
}

/// SplitMix64: a small, deterministic generator (no dependency, same results on every platform).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in 0..1.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in -1..1.
    fn signed(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }
}

/// The brightness `src` gives at `(x, y)`: the mean of its colour channels (any colour model;
/// only its gradient's direction is used, so the sign convention of CMYK does not matter).
fn brightness(src: &Surface, fmt: &PixelFormat, x: i32, y: i32) -> f32 {
    let n = fmt.mode.color_channels().max(1);
    let sum: f32 = (0..n).map(|c| src.sample_channel(x, y, c)).sum();
    sum / n as f32
}

/// The source's contour direction at `(x, y)`: a unit vector across its brightness gradient, or
/// `None` where the source is flat.
fn contour(src: &Surface, fmt: &PixelFormat, x: f32, y: f32, reach: i32) -> Option<(f32, f32)> {
    let (xi, yi) = (x.round() as i32, y.round() as i32);
    let gx = brightness(src, fmt, xi.saturating_add(reach), yi) - brightness(src, fmt, xi.saturating_sub(reach), yi);
    let gy = brightness(src, fmt, xi, yi.saturating_add(reach)) - brightness(src, fmt, xi, yi.saturating_sub(reach));
    let len = (gx * gx + gy * gy).sqrt();
    (len.is_finite() && len > 1e-3).then(|| (-gy / len, gx / len))
}

/// Where the strokes are allowed to start: inside the canvas.
fn inside(b: Rect, x: f32, y: f32) -> bool {
    x.is_finite() && y.is_finite() && x >= b.x0 as f32 && y >= b.y0 as f32 && x < b.x1 as f32 && y < b.y1 as f32
}

/// The options that shape the strokes (from the params).
#[derive(Clone, Copy, Debug)]
struct Options {
    style: Style,
    area: f32,
    tolerance: f32,
    seed: u64,
}

/// Paint the stroke's art strokes onto `surf` from `src` (same format). Returns the damage and how
/// many strokes were painted.
#[allow(clippy::too_many_arguments)]
fn paint_art(
    surf: &mut Surface,
    src: &Surface,
    canvas: Rect,
    stroke: &Stroke,
    o: Options,
    sel: Option<&Surface>,
    lock: bool,
    mode: photocraft_color::BlendMode,
) -> (Rect, usize) {
    let fmt = surf.format();
    let ch = fmt.channels();
    let alpha = alpha_index(&fmt);
    let size = stroke.brush.size.max(1.0);
    let mut rng = Rng(o.seed);
    // Dab centres, thinned so the strokes don't pile up where the pointer lingers.
    let gap = (o.area * 0.35).max(size * 0.5).max(1.0);
    let mut centres: Vec<(f32, f32)> = Vec::new();
    for d in dabs(stroke) {
        let c = (d.center.x as f32, d.center.y as f32);
        if centres.last().is_none_or(|l| ((l.0 - c.0).powi(2) + (l.1 - c.1).powi(2)).sqrt() >= gap) {
            centres.push(c);
        }
    }
    let per_dab = (((o.area.max(size) / size).powi(2) * if o.style == Style::Dab { 0.9 } else { 0.6 }).round() as usize).clamp(1, MAX_PER_DAB);
    // The stroke tip: a little narrower than the brush, as each stroke is one mark of many.
    let tip = size * if o.style.tight() { 0.6 } else { 0.75 };
    let step = (tip * 0.3).max(1.0);
    let length = o.style.length() * tip;
    // Long strokes of a huge brush: keep each one's coverage bounded.
    let length = length.min((40_000_000.0 / tip.max(1.0)).max(step));
    let steps = (length / step).ceil().clamp(0.0, 400.0) as usize;
    let reach = (tip * 0.25).clamp(1.0, 8.0) as i32;
    let mut damage = Rect::EMPTY;
    let mut painted = 0usize;
    let mut spent = 0u64;
    let mut src_px = vec![0.0f32; ch];
    let mut dst_px = vec![0.0f32; ch];
    'dabs: for (cx, cy) in centres {
        for _ in 0..per_dab {
            if painted >= MAX_STROKES || spent >= PIXEL_BUDGET {
                break 'dabs;
            }
            // Where this stroke starts: anywhere in the disc of the area around the dab.
            let (r, a) = (rng.unit().sqrt() * o.area * 0.5 * o.style.spread(), rng.unit() * std::f32::consts::TAU);
            let (sx, sy) = (cx + r * a.cos(), cy + r * a.sin());
            let jitter = rng.signed() * o.style.jitter();
            let curl = o.style.curl() * if rng.unit() < 0.5 { -1.0 } else { 1.0 };
            let size_k = 0.7 + 0.3 * rng.unit();
            if !inside(canvas, sx, sy) {
                continue;
            }
            let (px, py) = (sx.floor() as i32, sy.floor() as i32);
            src.read_pixel(px, py, &mut src_px);
            if alpha.is_some_and(|a| src_px.get(a).is_some_and(|v| *v <= 0.0)) {
                continue;
            }
            // Tolerance: only where the source differs enough from what is there now.
            if o.tolerance > 0.0 {
                surf.read_pixel(px, py, &mut dst_px);
                let diff = src_px.iter().zip(&dst_px).map(|(s, d)| (s - d).abs()).fold(0.0f32, f32::max);
                if diff * 100.0 < o.tolerance {
                    continue;
                }
            }
            // The path: along the source's contours, turning by the style's curl.
            let mut dir = contour(src, &fmt, sx, sy, reach).unwrap_or_else(|| (a.cos(), a.sin()));
            dir = rotate(dir, jitter);
            let mut points = vec![StrokePoint::new(f64::from(sx), f64::from(sy), 1.0)];
            let (mut x, mut y) = (sx, sy);
            for _ in 0..steps {
                if let Some(c) = contour(src, &fmt, x, y, reach) {
                    // Keep going the same way: a contour has two directions.
                    let c = if c.0 * dir.0 + c.1 * dir.1 < 0.0 { (-c.0, -c.1) } else { c };
                    dir = normalise((dir.0 * 0.5 + c.0 * 0.5, dir.1 * 0.5 + c.1 * 0.5)).unwrap_or(dir);
                }
                dir = rotate(dir, curl);
                x += dir.0 * step;
                y += dir.1 * step;
                if !inside(canvas, x, y) {
                    break;
                }
                points.push(StrokePoint::new(f64::from(x), f64::from(y), 1.0));
            }
            let mut brush = stroke.brush.clone();
            brush.size = (tip * size_k).max(1.0);
            brush.spacing = brush.spacing.min(0.25);
            let mark = Stroke { brush, points };
            let (bounds, cov) = stroke_coverage(&mark);
            if bounds.is_empty() {
                continue;
            }
            let pixels = u64::from(bounds.width()) * u64::from(bounds.height());
            spent = spent.saturating_add(pixels);
            let mut paint = Region::new(bounds, ch);
            for px in paint.data.chunks_exact_mut(ch) {
                px.copy_from_slice(&src_px);
            }
            let d = apply_coverage(surf, bounds, &cov, stroke.brush.opacity, sel, lock, &paint, mode);
            damage = damage.union(&d);
            painted += 1;
        }
    }
    (damage, painted)
}

fn rotate((x, y): (f32, f32), a: f32) -> (f32, f32) {
    if a == 0.0 {
        return (x, y);
    }
    let (s, c) = a.sin_cos();
    (x * c - y * s, x * s + y * c)
}

fn normalise((x, y): (f32, f32)) -> Option<(f32, f32)> {
    let l = (x * x + y * y).sqrt();
    (l.is_finite() && l > 1e-6).then(|| (x / l, y / l))
}

/// A seed from the stroke itself, so a replayed or journaled stroke paints the same strokes.
fn stroke_seed(stroke: &Stroke) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in stroke.points.iter().take(64) {
        for v in [p.x.to_bits(), p.y.to_bits()] {
            h ^= v;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h ^ stroke.points.len() as u64
}

pub(super) fn art_history_brush(s: &mut Session, p: &Value) -> Result<Value> {
    let (stroke, id) = parse_brush(s, p, CMD)?;
    let mode = blend_param(p, CMD)?;
    let style = Style::parse(p)?;
    let area = num(p, "area", 50.0);
    if !area.is_finite() || !(0.0..=500.0).contains(&area) {
        return Err(bad(CMD, "`area` must be 0..500 px"));
    }
    let tolerance = num(p, "tolerance", 0.0);
    if !tolerance.is_finite() || !(0.0..=100.0).contains(&tolerance) {
        return Err(bad(CMD, "`tolerance` must be 0..100 %"));
    }
    let seed = match p.get("seed") {
        None => stroke_seed(&stroke),
        Some(v) => v.as_u64().ok_or_else(|| bad(CMD, "`seed` must be a whole number"))?,
    };
    let canvas = s.active().map(|st| st.doc.bounds()).unwrap_or(Rect::EMPTY);
    let src_surface = history_source(s, id, p, CMD)?;
    let o = Options { style, area, tolerance, seed };
    let mut count = 0usize;
    let dmg = run_stroke(s, "Art History Brush", id, p, |_, surf, sel, lock| {
        let fmt = surf.format();
        let src = if src_surface.format() == fmt { src_surface } else { src_surface.convert(fmt) };
        let (d, n) = paint_art(surf, &src, canvas, &stroke, o, sel, lock, mode);
        count = n;
        Ok(d)
    })?;
    Ok(json!({ "damage": damage_json(dmg), "strokes": count }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_parse_by_name_and_reject_others() {
        for (s, n) in Style::ALL {
            assert_eq!(Style::parse(&json!({ "style": n })).unwrap(), s);
        }
        assert_eq!(Style::parse(&json!({})).unwrap(), Style::TightShort);
        assert!(Style::parse(&json!({"style": "bogus"})).is_err());
        assert!(Style::parse(&json!({"style": 3})).is_err());
    }

    #[test]
    fn the_generator_is_deterministic_and_in_range() {
        let (mut a, mut b) = (Rng(7), Rng(7));
        for _ in 0..1000 {
            let (x, y) = (a.unit(), b.unit());
            assert_eq!(x.to_bits(), y.to_bits());
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn contour_runs_across_the_gradient() {
        // A horizontal ramp: brightness rises with x, so the contours are vertical.
        let fmt = PixelFormat::RGBA8;
        let mut s = Surface::new(fmt);
        let b = Rect::new(0, 0, 32, 32);
        let mut data = Vec::new();
        for _y in 0..32 {
            for x in 0..32 {
                let v = x as f32 / 31.0;
                data.extend([v, v, v, 1.0]);
            }
        }
        s.write_region(b, &data);
        let (dx, dy) = contour(&s, &fmt, 16.0, 16.0, 2).unwrap();
        assert!(dx.abs() < 1e-3 && (dy.abs() - 1.0).abs() < 1e-3, "({dx}, {dy})");
        let flat = Surface::new(fmt);
        assert!(contour(&flat, &fmt, 4.0, 4.0, 2).is_none());
    }
}
