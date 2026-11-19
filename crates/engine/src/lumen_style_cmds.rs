//! Lumen's quick styles (Image › Quick Styles, "Estilos rápidos" in Spanish): one-click looks built from ordinary adjustment
//! layers, so every step stays editable and the whole style undoes as one history step.
//!
//! Also Lumen's Camera Raw looks ([`LOOKS`]): one Camera Raw pass with develop settings of
//! Lumen's own, which stay private (see [`Look`]).

use photocraft_algo::camera_raw::{CameraRaw, Wheel};
use photocraft_doc::{Layer, LayerContent};
use photocraft_geom::Rect;
use photocraft_raster::{Surface, from_rgba_into};
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session, adjust_params};

/// One adjustment layer of a style: (layer name, adjustment kind, its params).
type Step = (&'static str, &'static str, &'static str);

/// The styles: (command id, menu label, Spanish name for the history step, steps bottom to top).
/// Menu labels are English source strings, translated by the UI catalogs like every other menu
/// item; layer and history names are document data, so they stay in Spanish.
pub const STYLES: &[(&str, &str, &str, &[Step])] = &[
    (
        "image.lumenStyle.warm",
        "Warm",
        "Cálido",
        &[("Cálido · Filtro", "photoFilter", r##"{"color":"#ec8a00","density":30}"##), ("Cálido · Intensidad", "vibrance", r##"{"vibrance":15}"##)],
    ),
    (
        "image.lumenStyle.cool",
        "Cool",
        "Frío",
        &[("Frío · Filtro", "photoFilter", r##"{"color":"#0b8ed6","density":25}"##), ("Frío · Intensidad", "vibrance", r##"{"vibrance":10}"##)],
    ),
    (
        "image.lumenStyle.vintage",
        "Vintage",
        "Vintage",
        &[
            ("Vintage · Curvas", "curves", r##"{"points":[[0,30],[128,128],[255,230]]}"##),
            ("Vintage · Saturación", "hueSaturation", r##"{"saturation":-25}"##),
            ("Vintage · Filtro", "photoFilter", r##"{"color":"#c8a165","density":20}"##),
        ],
    ),
    (
        "image.lumenStyle.bw",
        "High-Contrast B&W",
        "B/N intenso",
        &[("B/N · Blanco y negro", "blackWhite", "{}"), ("B/N · Contraste", "curves", r##"{"points":[[0,0],[64,44],[192,212],[255,255]]}"##)],
    ),
    (
        "image.lumenStyle.cinema",
        "Cinema",
        "Cine",
        &[
            ("Cine · Equilibrio de color", "colorBalance", r##"{"shadows":[-10,0,15],"highlights":[10,0,-12]}"##),
            ("Cine · Curvas", "curves", r##"{"points":[[0,8],[64,56],[192,200],[255,248]]}"##),
            ("Cine · Intensidad", "vibrance", r##"{"vibrance":-10}"##),
        ],
    ),
    (
        "image.lumenStyle.pop",
        "Pop",
        "Pop",
        &[("Pop · Intensidad", "vibrance", r##"{"vibrance":35,"saturation":10}"##), ("Pop · Contraste", "brightnessContrast", r##"{"contrast":15}"##)],
    ),
];

/// A Camera Raw look (Image › Quick Styles): what adjustment layers can't do (the eight-band
/// Color Mixer, the Color Grading wheels, sharpening and noise reduction) in one Camera Raw pass.
///
/// Its settings are Lumen's and stay private. They aren't in the source (the repository is
/// public): the build reads them from an environment variable, a GitHub Actions secret for the
/// releases (see [`parse_settings`]); a build without it doesn't offer the look. The document
/// records only the look's id (on a Smart Object, a Smart Filter whose params hold just the
/// document's transfer curve, so double-clicking it opens no dialog), and the History step
/// carries the look's name. A look can be offered for a limited time (`until`); documents that
/// used it keep rendering it afterwards.
pub struct Look {
    /// Command id, also the Smart Filter's command.
    pub id: &'static str,
    /// Menu label (English source; the UI catalogs translate it).
    pub label: &'static str,
    /// History step name (document data, in Spanish like the other styles).
    pub history: &'static str,
    /// Offered until this moment, in Unix seconds (UTC); `None`: always.
    pub until: Option<u64>,
    /// `until` for people, in the message once it has passed.
    pub until_text: &'static str,
    /// The settings this build was given, if any.
    settings: fn() -> Option<LookSettings>,
}

/// A look's settings: one Camera Raw pass, then a glow.
#[derive(Clone, Debug, PartialEq)]
pub struct LookSettings {
    pub raw: CameraRaw,
    /// Glow strength, 0–100 % (0: none). See [`glow_surface`].
    pub glow: f32,
    /// Glow spread, in % of the image's short side (0–20).
    pub glow_radius: f32,
}

impl Look {
    /// Whether this build has the look's settings.
    pub fn available(&self) -> bool {
        (self.settings)().is_some()
    }
}

/// 13 December 2026 at 24:00 in Spain (CET, UTC+1), i.e. 2026-12-13T23:00:00Z.
pub const TROPICAL_SUNSET_UNTIL: u64 = 1_797_202_800;

pub const LOOKS: &[Look] = &[Look {
    id: "image.lumenStyle.tropicalSunset",
    label: "Tropical Sunset",
    history: "Estilo Atardecer tropical",
    until: Some(TROPICAL_SUNSET_UNTIL),
    until_text: "13 December 2026",
    settings: tropical_sunset,
}];

/// The Tropical Sunset settings, from `LUMEN_LOOK_TROPICAL_SUNSET` at build time (see
/// [`parse_settings`]). Tests use stand-in values when the build has none.
const TROPICAL_SUNSET_SETTINGS: Option<&str> = match option_env!("LUMEN_LOOK_TROPICAL_SUNSET") {
    Some(v) => Some(v),
    None if cfg!(test) => Some(TEST_SETTINGS),
    None => None,
};

/// Stand-in settings for tests (not the look's).
const TEST_SETTINGS: &str = "20,-30,10,10,-20,30,0, 0,0,0,0,0,0,0,0, 10,10,10,-20,20,10,10,20, 0,10,0,0,0,0,0,0, 200,20,0,320,10,0,30,20,0, 30,1,25,50, 5,25";

/// Warm pink-orange skies, teal shadows and lifted colour: a tropical sunset look.
fn tropical_sunset() -> Option<LookSettings> {
    parse_settings(TROPICAL_SUNSET_SETTINGS?)
}

/// A look's settings from its build-time variable: 46 numbers separated by commas (spaces
/// allowed), in Lightroom's panel order: Contrast, Highlights, Shadows, Whites, Blacks, Vibrance,
/// Saturation; the Color Mixer's Hue, then Saturation, then Luminance, each for red, orange,
/// yellow, green, aqua, blue, purple and magenta; Color Grading's Shadows, Midtones and
/// Highlights wheels as hue, saturation, luminance; Sharpening amount, radius, detail and
/// masking; Noise Reduction luminance and color.
///
/// Nine more may follow, for the panels a preset leaves to each photo or doesn't show: Exposure,
/// Temperature, Tint, Dehaze, Clarity, and the Tone Curve's Shadows, Darks, Lights and Highlights;
/// then three more: Vignette amount, Glow (0–100 %) and Glow radius (0–20 % of the short side).
/// Left out, they are 0. `None` if anything is missing or out of range.
pub fn parse_settings(text: &str) -> Option<LookSettings> {
    let v: Vec<f32> = text.split(',').map(|t| t.trim().parse::<f32>().ok().filter(|x| x.is_finite())).collect::<Option<_>>()?;
    // Each slider's range, in the order above (Camera Raw's own check covers only its curves).
    let pm = (-100.0, 100.0);
    let mut ranges = vec![pm; 7 + 24];
    for _ in 0..3 {
        ranges.extend([(0.0, 360.0), (0.0, 100.0), pm]);
    }
    ranges.extend([(0.0, 150.0), (0.5, 3.0), (0.0, 100.0), (0.0, 100.0), (0.0, 100.0), (0.0, 100.0)]);
    let shown = ranges.len();
    ranges.push((-5.0, 5.0));
    ranges.extend([pm; 8]);
    let developed = ranges.len();
    ranges.extend([pm, (0.0, 100.0), (0.0, 20.0)]);
    if ![shown, developed, ranges.len()].contains(&v.len()) || !v.iter().zip(&ranges).all(|(x, (lo, hi))| (*lo..=*hi).contains(x)) {
        return None;
    }
    let (v, extra) = v.split_at(shown);
    let (extra, finish) = extra.split_at(extra.len().min(9));
    let [exposure, temperature, tint, dehaze, clarity, curve_shadows, curve_darks, curve_lights, curve_highlights] = match extra {
        [] => [0.0; 9],
        e => <[f32; 9]>::try_from(e).ok()?,
    };
    let [vignette, glow, glow_radius] = match finish {
        [] => [0.0; 3],
        e => <[f32; 3]>::try_from(e).ok()?,
    };
    let [contrast, highlights, shadows, whites, blacks, vibrance, saturation, rest @ ..] = v else { return None };
    let (hue, rest) = rest.split_first_chunk::<8>()?;
    let (sat, rest) = rest.split_first_chunk::<8>()?;
    let (lum, rest) = rest.split_first_chunk::<8>()?;
    let [sh, ss, sl, mh, ms, ml, hh, hs, hl, amount, radius, detail, masking, noise_lum, noise_color] = rest else { return None };
    let cr = CameraRaw {
        exposure,
        temperature,
        tint,
        dehaze,
        clarity,
        curve_shadows,
        curve_darks,
        curve_lights,
        curve_highlights,
        contrast: *contrast,
        highlights: *highlights,
        shadows: *shadows,
        whites: *whites,
        blacks: *blacks,
        vibrance: *vibrance,
        saturation: *saturation,
        hsl_hue: *hue,
        hsl_sat: *sat,
        hsl_lum: *lum,
        grade_shadows: Wheel { hue: *sh, sat: *ss, lum: *sl },
        grade_midtones: Wheel { hue: *mh, sat: *ms, lum: *ml },
        grade_highlights: Wheel { hue: *hh, sat: *hs, lum: *hl },
        sharpen_amount: *amount,
        sharpen_radius: *radius,
        sharpen_detail: *detail,
        sharpen_masking: *masking,
        noise_luminance: *noise_lum,
        noise_color: *noise_color,
        vignette_amount: vignette,
        ..CameraRaw::default()
    };
    cr.validate().ok()?;
    Some(LookSettings { raw: cr, glow, glow_radius })
}

/// What a glow takes from the image: the part of a channel above this level (display values).
const GLOW_THRESHOLD: f32 = 0.62;

/// Glow (bloom): the parts of `surf` brighter than [`GLOW_THRESHOLD`], blurred over
/// `radius_pct` % of the area's short side and screened back over it at `amount` %, the soft light
/// bleed of a game's sun and neon. Works at any depth and on Gray (through RGBA); alpha is kept.
/// The bright pass is blurred on a quarter-size grid: the glow is soft, so that loses nothing.
pub fn glow_surface(surf: &Surface, area: Rect, amount: f32, radius_pct: f32) -> Surface {
    let k = if amount.is_finite() { (amount / 100.0).clamp(0.0, 1.0) } else { 0.0 };
    let area = area.intersect(&surf.content_bounds().union(&area));
    let (w, h) = (area.width() as usize, area.height() as usize);
    let len = w.saturating_mul(h);
    if k <= 0.0 || len == 0 || len as u64 > crate::lens_cmds::MAX_RAW_PIXELS {
        return surf.clone();
    }
    let mut px = vec![[0.0f32; 4]; len];
    surf.read_rgba_into(area, &mut px);
    let f = 4usize.min(w.min(h)).max(1);
    let (sw, sh) = (w.div_ceil(f), h.div_ceil(f));
    let mut planes = [vec![0.0f32; sw * sh], vec![0.0f32; sw * sh], vec![0.0f32; sw * sh]];
    let mut count = vec![0.0f32; sw * sh];
    for (i, p) in px.iter().enumerate() {
        let j = (i / w / f) * sw + (i % w) / f;
        for (plane, v) in planes.iter_mut().zip(p) {
            plane[j] += ((v - GLOW_THRESHOLD) / (1.0 - GLOW_THRESHOLD)).clamp(0.0, 1.0);
        }
        count[j] += 1.0;
    }
    let radius_pct = if radius_pct.is_finite() { radius_pct.clamp(0.0, 20.0) } else { 0.0 };
    let sigma = w.min(h) as f32 * radius_pct / 100.0 / f as f32;
    // Three box passes approximate a Gaussian of `sigma`.
    let r = (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) / 2.0).round().max(0.0) as usize;
    for plane in &mut planes {
        for (v, n) in plane.iter_mut().zip(&count) {
            *v /= n.max(1.0);
        }
        for _ in 0..3 {
            box_blur(plane, sw, sh, r);
        }
    }
    for (i, p) in px.iter_mut().enumerate() {
        let fx = (((i % w) as f32 + 0.5) / f as f32 - 0.5).clamp(0.0, (sw - 1) as f32);
        let fy = (((i / w) as f32 + 0.5) / f as f32 - 0.5).clamp(0.0, (sh - 1) as f32);
        let (x0, y0) = (fx as usize, fy as usize);
        let (x1, y1) = ((x0 + 1).min(sw - 1), (y0 + 1).min(sh - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        for (c, plane) in planes.iter().enumerate() {
            let top = plane[y0 * sw + x0] * (1.0 - tx) + plane[y0 * sw + x1] * tx;
            let bottom = plane[y1 * sw + x0] * (1.0 - tx) + plane[y1 * sw + x1] * tx;
            let g = (top * (1.0 - ty) + bottom * ty) * k;
            // Screen: lifts toward white, never past it.
            p[c] += g * (1.0 - p[c].clamp(0.0, 1.0));
        }
    }
    let fmt = surf.format();
    let n = fmt.channels();
    let orig = surf.read_region(area);
    let mut data = vec![0.0f32; len * n];
    for (i, (q, o)) in px.iter().zip(data.chunks_exact_mut(n)).enumerate() {
        from_rgba_into(&fmt, *q, o);
        if fmt.alpha
            && let (Some(dst), Some(src)) = (o.last_mut(), orig.get(i * n + n - 1))
        {
            *dst = *src;
        }
    }
    let mut out = surf.clone();
    out.write_region(area, &data);
    out.prune();
    out
}

/// One box-blur pass of radius `r` over a `w`×`h` plane, edges clamped.
fn box_blur(plane: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 || w == 0 || h == 0 || plane.len() != w * h {
        return;
    }
    let norm = 1.0 / (2 * r + 1) as f32;
    let ri = r as isize;
    let mut tmp = vec![0.0f32; plane.len()];
    for (row, out) in plane.chunks_exact(w).zip(tmp.chunks_exact_mut(w)) {
        let at = |i: isize| row[i.clamp(0, w as isize - 1) as usize];
        let mut acc: f32 = (-ri..=ri).map(at).sum();
        for (x, o) in out.iter_mut().enumerate() {
            *o = acc * norm;
            acc += at(x as isize + ri + 1) - at(x as isize - ri);
        }
    }
    for x in 0..w {
        let at = |i: isize| tmp[i.clamp(0, h as isize - 1) as usize * w + x];
        let mut acc: f32 = (-ri..=ri).map(at).sum();
        for y in 0..h {
            plane[y * w + x] = acc * norm;
            acc += at(y as isize + ri + 1) - at(y as isize - ri);
        }
    }
}

/// The look's pixels: Camera Raw over `area`, then its glow.
fn develop(surf: &Surface, area: Rect, ls: &LookSettings) -> Surface {
    let raw = crate::lens_cmds::camera_raw_surface(surf, area, &ls.raw);
    glow_surface(&raw, area, ls.glow, ls.glow_radius)
}

fn look(id: &str) -> Option<&'static Look> {
    LOOKS.iter().find(|l| l.id == id)
}

/// Seconds since 1970 (UTC), or `None` where there is no clock (the web build).
fn unix_now() -> Option<u64> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// Whether command `id` is offered at `now`: every command except a look this build has no
/// settings for, or one past its `until` (or with no clock to tell, `now == None`).
pub fn offered_at(id: &str, now: Option<u64>) -> bool {
    let Some(l) = look(id) else { return true };
    l.available() && l.until.is_none_or(|until| now.is_some_and(|t| t < until))
}

/// [`offered_at`] now. The UI leaves a look out of the menus once this is `false`.
pub fn offered(id: &str) -> bool {
    offered_at(id, unix_now())
}

/// The look's settings for a document with transfer curve `gamma` (see `lens_cmds::encoding_gamma`).
fn look_settings(l: &Look, gamma: f32) -> Option<LookSettings> {
    let mut ls = (l.settings)()?;
    ls.raw.encoding_gamma = if gamma.is_finite() && gamma > 0.0 { gamma } else { 0.0 };
    Some(ls)
}

/// Apply look `id` to the active layer, if it is offered at `now`. Returns the layer.
pub fn apply_look_at(s: &mut Session, id: &str, now: Option<u64>) -> Result<Value> {
    let l = look(id).ok_or_else(|| bad(id, "unknown style"))?;
    if !l.available() {
        return Err(bad(id, "this style isn't included in this build"));
    }
    if !offered_at(id, now) {
        return Err(bad(id, format!("this limited-time style was available until {}", l.until_text)));
    }
    crate::lens_cmds::raw_enabled(s).map_err(|e| bad(id, e))?;
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let gamma = crate::lens_cmds::encoding_gamma(&d.doc);
    // As Camera Raw: one pass over the canvas plus the layer's content, refused beyond its limit.
    let layer = d.active_layer.ok_or_else(|| bad(id, "no active layer"))?;
    let area = d.doc.layer(layer).and_then(Layer::surface).map_or(d.doc.bounds(), |sf| d.doc.bounds().union(&sf.content_bounds()));
    if u64::from(area.width()).saturating_mul(u64::from(area.height())) > crate::lens_cmds::MAX_RAW_PIXELS {
        return Err(bad(id, format!("the image is too large for this style ({}×{} px)", area.width(), area.height())));
    }
    let ls = look_settings(l, gamma).ok_or_else(|| bad(id, "this style isn't included in this build"))?;
    let stored = json!({ "encodingGamma": (f64::from(gamma) * 1e4).round() / 1e4 });
    let layer = crate::lens_cmds::run_filter(s, l.id, l.history, stored, false, &|surf, canvas| develop(surf, canvas.union(&surf.content_bounds()), &ls))?;
    Ok(json!({ "style": l.history, "layer": layer.0 }))
}

/// Smart-filter hook: re-renders a look stored on a Smart Object (`None` for other ids), also
/// after it stops being offered, so documents keep their pixels.
pub fn look_surface(id: &str, params: &Value, surf: &Surface, canvas: Rect) -> Option<Surface> {
    let l = look(id)?;
    let gamma = params.get("encodingGamma").and_then(Value::as_f64).filter(|g| g.is_finite() && (0.0..=10.0).contains(g)).unwrap_or(0.0);
    Some(develop(surf, canvas, &look_settings(l, gamma as f32)?))
}

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

/// Add the style `id`'s adjustment layers above the active layer, as one history step. Returns
/// the new layers' ids, bottom to top; the top one becomes active.
pub fn apply(s: &mut Session, id: &str) -> Result<Value> {
    let (_, _, name, steps) = STYLES.iter().find(|(sid, ..)| *sid == id).ok_or_else(|| bad(id, "unknown style"))?;
    let mode = s.active().ok_or(EngineError::NoDocument)?.doc.mode;
    let mut layers = Vec::with_capacity(steps.len());
    for (name, kind, params) in steps.iter() {
        let p: Value = serde_json::from_str(params).map_err(|e| bad(id, format!("style step {name}: {e}")))?;
        layers.push((*name, adjust_params::from_params(kind, &p, None, mode)?));
    }
    let ids = s.edit(&format!("Estilo {name}"), |doc, active| {
        let mut ids = Vec::with_capacity(layers.len());
        for (name, adj) in layers {
            let l = Layer::new(doc.next_layer_name(name), LayerContent::Adjustment(adj));
            let new_id = doc.insert_above(*active, l);
            *active = Some(new_id);
            ids.push(new_id.0);
        }
        Ok(ids)
    })?;
    Ok(json!({ "style": name, "layers": ids }))
}

/// The command for style `index`, run by `run` (one fn per style: a command's `run` gets no id).
fn spec(index: usize, run: fn(&mut Session, &Value) -> Result<Value>) -> Option<CommandSpec> {
    let &(id, label, ..) = STYLES.get(index)?;
    Some(CommandSpec { id, label, menu: &["Image", "Quick Styles"], shortcut: None, params: "{}", enabled: has_doc, run, journal: true })
}

macro_rules! style_run {
    ($index:literal) => {{
        fn run(s: &mut Session, _: &Value) -> Result<Value> {
            let id = STYLES.get($index).map(|st| st.0).ok_or_else(|| bad("image.lumenStyle", "unknown style"))?;
            apply(s, id)
        }
        ($index, run as fn(&mut Session, &Value) -> Result<Value>)
    }};
}

/// The commands of look `index` (one pair of fns per look: a command's `run` and `enabled` get no id).
macro_rules! look_cmd {
    ($index:literal) => {{
        fn id() -> std::result::Result<&'static str, String> {
            LOOKS.get($index).map(|l| l.id).ok_or_else(|| "unknown style".to_string())
        }
        fn run(s: &mut Session, _: &Value) -> Result<Value> {
            let id = id().map_err(|e| bad("image.lumenStyle", e))?;
            apply_look_at(s, id, unix_now())
        }
        fn enabled(s: &Session) -> std::result::Result<(), String> {
            if !offered(id()?) {
                return Err("this limited-time style is no longer available".into());
            }
            crate::lens_cmds::raw_enabled(s)
        }
        ($index, run as fn(&mut Session, &Value) -> Result<Value>, enabled as fn(&Session) -> std::result::Result<(), String>)
    }};
}

pub fn specs() -> Vec<CommandSpec> {
    let styles = [style_run!(0), style_run!(1), style_run!(2), style_run!(3), style_run!(4), style_run!(5)].into_iter().filter_map(|(i, run)| spec(i, run));
    let looks = [look_cmd!(0)].into_iter().filter_map(|(i, run, enabled)| {
        let l = LOOKS.get(i)?;
        Some(CommandSpec { id: l.id, label: l.label, menu: &["Image", "Quick Styles"], shortcut: None, params: "{}", enabled, run, journal: true })
    });
    styles.chain(looks).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 16, "height": 16, "background": "white"})).unwrap();
        s
    }

    #[test]
    fn every_style_has_a_command() {
        assert_eq!(specs().len(), STYLES.len() + LOOKS.len());
        for id in STYLES.iter().map(|s| s.0).chain(LOOKS.iter().map(|l| l.id)) {
            assert!(crate::command_specs().iter().any(|c| c.id == id), "{id} isn't registered");
        }
    }

    #[test]
    fn each_style_adds_its_layers_in_one_undo_step() {
        for (id, _, _, steps) in STYLES {
            let mut s = session();
            let before = s.active().unwrap().doc.layer_count();
            let r = s.execute(id, json!({})).unwrap();
            assert_eq!(r["layers"].as_array().map(Vec::len), Some(steps.len()), "{id}");
            assert_eq!(s.active().unwrap().doc.layer_count(), before + steps.len(), "{id}");
            s.execute("edit.undo", json!({})).unwrap();
            assert_eq!(s.active().unwrap().doc.layer_count(), before, "{id}: one undo removes the whole style");
        }
    }

    #[test]
    fn styles_fail_cleanly_without_a_document() {
        let mut s = Session::new();
        for (id, ..) in STYLES {
            assert!(s.execute(id, json!({})).is_err(), "{id}");
            assert!(s.execute(id, json!("nonsense")).is_err(), "{id}");
        }
        assert!(apply(&mut session(), "image.lumenStyle.nope").is_err());
    }

    /// A fixed moment before the cutoff, so the tests don't depend on today's date.
    const BEFORE: Option<u64> = Some(TROPICAL_SUNSET_UNTIL - 86_400);
    const AFTER: Option<u64> = Some(TROPICAL_SUNSET_UNTIL);

    fn photo() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 24, "height": 16, "background": "white"})).unwrap();
        s.execute("layer.new.layer", json!({"name": "photo"})).unwrap();
        s.edit("photo", |doc, active| {
            let surf = doc.layer_mut(active.unwrap()).unwrap().surface_mut().unwrap();
            let fmt = surf.format();
            for y in 0..16 {
                for x in 0..24 {
                    // A sky-to-street gradient: warm at the top, blue-green below.
                    let t = y as f32 / 15.0;
                    surf.write_pixel(x, y, &photocraft_raster::from_rgba(&fmt, [0.9 - 0.6 * t, 0.5, 0.4 + 0.3 * t, 1.0]));
                }
            }
            Ok(())
        })
        .unwrap();
        s
    }

    fn pixels(s: &Session) -> Vec<[f32; 4]> {
        let d = s.active().unwrap();
        let surf = d.doc.layer(d.active_layer.unwrap()).and_then(Layer::surface).unwrap();
        let mut px = vec![[0.0f32; 4]; 24 * 16];
        surf.read_rgba_into(Rect::new(0, 0, 24, 16), &mut px);
        px
    }

    #[test]
    fn settings_parse_in_lightrooms_order_and_reject_bad_lists() {
        let cr = parse_settings(TEST_SETTINGS).unwrap().raw;
        assert_eq!((cr.contrast, cr.highlights, cr.saturation), (20.0, -30.0, 0.0));
        assert_eq!(cr.hsl_sat, [10.0, 10.0, 10.0, -20.0, 20.0, 10.0, 10.0, 20.0]);
        assert_eq!(cr.hsl_lum[1], 10.0);
        assert_eq!((cr.grade_shadows.hue, cr.grade_midtones.hue, cr.grade_highlights.sat), (200.0, 320.0, 20.0));
        assert_eq!((cr.sharpen_amount, cr.sharpen_radius, cr.sharpen_masking, cr.noise_color), (30.0, 1.0, 50.0, 25.0));
        let short = TEST_SETTINGS.rsplit_once(',').unwrap().0;
        for bad in ["", "1,2,3", short, &format!("{TEST_SETTINGS},1"), &TEST_SETTINGS.replacen("20", "x", 1), &TEST_SETTINGS.replacen("20", "NaN", 1)] {
            assert!(parse_settings(bad).is_none(), "{bad}");
        }
        assert!(parse_settings(&TEST_SETTINGS.replacen("20", "900", 1)).is_none(), "out of Camera Raw's range");
        // The nine optional per-photo / unshown values: all of them or none.
        let cr = parse_settings(&format!("{TEST_SETTINGS}, -0.5,-10,15,20,5,-20,10,15,10")).unwrap().raw;
        assert_eq!((cr.exposure, cr.temperature, cr.tint, cr.dehaze, cr.clarity), (-0.5, -10.0, 15.0, 20.0, 5.0));
        assert_eq!((cr.curve_shadows, cr.curve_darks, cr.curve_lights, cr.curve_highlights), (-20.0, 10.0, 15.0, 10.0));
        assert_eq!(parse_settings(TEST_SETTINGS).unwrap().raw.exposure, 0.0);
        // Then vignette, glow and glow radius.
        let ls = parse_settings(&format!("{TEST_SETTINGS}, 0,0,0,0,0,0,0,0,0, -10,40,2")).unwrap();
        assert_eq!((ls.raw.vignette_amount, ls.glow, ls.glow_radius), (-10.0, 40.0, 2.0));
        assert_eq!(parse_settings(TEST_SETTINGS).unwrap().glow, 0.0);
        assert!(parse_settings(&format!("{TEST_SETTINGS}, 0,0,0,0,0,0,0,0,0, -10,40")).is_none());
        assert!(parse_settings(&format!("{TEST_SETTINGS}, 0,0,0,0,0,0,0,0,0, 0,150,3")).is_none(), "glow out of range");
        assert!(parse_settings(&format!("{TEST_SETTINGS}, -0.5,-10")).is_none(), "some but not all");
        assert!(parse_settings(&format!("{TEST_SETTINGS}, -9,0,0,0,0,0,0,0,0")).is_none(), "exposure out of range");
    }

    #[test]
    fn the_look_is_offered_until_13_december_2026_in_spain() {
        let id = LOOKS[0].id;
        // 2026-12-13T22:59:59Z is 23:59:59 that day in Spain.
        assert!(offered_at(id, Some(1_797_202_799)));
        assert!(!offered_at(id, Some(1_797_202_800)), "from 14 December, 00:00 in Spain");
        assert!(!offered_at(id, None), "no clock: not offered");
        assert!(offered_at("image.lumenStyle.warm", None), "other commands are always offered");
    }

    #[test]
    fn the_look_changes_pixels_in_one_undo_step_and_stores_no_settings() {
        let id = LOOKS[0].id;
        let mut s = photo();
        let before = pixels(&s);
        let r = apply_look_at(&mut s, id, BEFORE).unwrap();
        assert_eq!(r["style"], "Estilo Atardecer tropical");
        let after = pixels(&s);
        assert!(before.iter().zip(&after).any(|(a, b)| a.iter().zip(b).any(|(x, y)| (x - y).abs() > 1e-3)), "the look changes the colours");
        s.execute("edit.undo", json!({})).unwrap();
        assert_eq!(pixels(&s), before, "one undo removes it");

        // On a Smart Object it is a Smart Filter named by the look, with no develop settings in it.
        let mut s = photo();
        s.execute("layer.smartObjects.convertToSmartObject", json!({})).unwrap();
        apply_look_at(&mut s, id, BEFORE).unwrap();
        let d = s.active().unwrap();
        let LayerContent::Smart(sm) = &d.doc.layer(d.active_layer.unwrap()).unwrap().content else { panic!("not a smart object") };
        let f = sm.smart_filters.last().unwrap();
        assert_eq!(f.command, id);
        let keys: Vec<&String> = f.params.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["encodingGamma"], "only the document's transfer curve is stored");
        // It still renders once the look stops being offered.
        let surf = Surface::new(photocraft_color::PixelFormat::RGBA8);
        assert!(look_surface(id, &f.params, &surf, Rect::new(0, 0, 4, 4)).is_some());
        assert!(look_surface("image.lumenStyle.nope", &json!({}), &surf, Rect::new(0, 0, 4, 4)).is_none());
    }

    #[test]
    fn after_the_cutoff_the_look_is_refused_with_a_clear_message() {
        let id = LOOKS[0].id;
        let mut s = photo();
        let before = pixels(&s);
        let e = apply_look_at(&mut s, id, AFTER).unwrap_err().to_string();
        assert!(e.contains("13 December 2026"), "{e}");
        assert_eq!(pixels(&s), before);
        assert!(apply_look_at(&mut Session::new(), id, BEFORE).is_err(), "no document");
        assert!(apply_look_at(&mut photo(), "image.lumenStyle.nope", BEFORE).is_err());
    }

    #[test]
    fn glow_lights_up_around_bright_parts_and_keeps_the_rest() {
        let fmt = photocraft_color::PixelFormat::RGBA8;
        let mut surf = Surface::new(fmt);
        let b = Rect::new(0, 0, 64, 64);
        let mut data = Vec::new();
        for y in 0..64 {
            for x in 0..64 {
                // A white square in a dark grey field, half transparent at the right.
                let v = if (24..40).contains(&x) && (24..40).contains(&y) { 1.0 } else { 0.2 };
                data.extend([v, v, v, if x >= 60 { 0.5 } else { 1.0 }]);
            }
        }
        surf.write_region(b, &data);
        let px = |s: &Surface, x, y| {
            let mut p = [0.0f32; 4];
            s.read_pixel(x, y, &mut p);
            p
        };
        let out = glow_surface(&surf, b, 60.0, 10.0);
        assert!(px(&out, 20, 32)[0] > px(&surf, 20, 32)[0] + 0.02, "light spills next to the square");
        assert!((px(&out, 2, 2)[0] - 0.2).abs() < 0.01, "far away stays put");
        assert!((px(&out, 62, 2)[3] - 0.5).abs() < 0.01, "alpha is kept");
        assert!(px(&out, 32, 32)[0] <= 1.0 + 1e-6, "never past white");
        assert_eq!(glow_surface(&surf, b, 0.0, 10.0).read_region(b), surf.read_region(b), "no glow at 0 %");
        assert_eq!(glow_surface(&surf, b, f32::NAN, f32::NAN).read_region(b), surf.read_region(b));
        let empty = Surface::new(fmt);
        assert!(glow_surface(&empty, Rect::EMPTY, 50.0, 5.0).content_bounds().is_empty());
    }
}
