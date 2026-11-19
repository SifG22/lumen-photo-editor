//! The Color panel's modes, chosen from its panel menu as in Photoshop: Hue Cube (the spectrum
//! field, the default), Color Wheel, and the Grayscale, RGB, HSB, CMYK, Lab and Web Color sliders,
//! each with the colour ramp under it. Every edit sets the foreground colour, or the background one
//! while its chip is framed, sRGB-encoded as the rest of the shell keeps tool colours.
//!
//! The panel menu also has Photoshop's Copy Color as HTML and Copy Color's Hex Code.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::PhotocraftApp;
use crate::state::ColorPanelMode;
use crate::theme::Tokens;
use crate::widgets;

/// One slider of a slider mode: its label, range and unit.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Channel {
    label: &'static str,
    lo: f32,
    hi: f32,
    suffix: &'static str,
}

const fn ch(label: &'static str, lo: f32, hi: f32, suffix: &'static str) -> Channel {
    Channel { label, lo, hi, suffix }
}

const GRAY: [Channel; 1] = [ch("K", 0.0, 100.0, "%")];
const RGB: [Channel; 3] = [ch("R", 0.0, 255.0, ""), ch("G", 0.0, 255.0, ""), ch("B", 0.0, 255.0, "")];
const HSB: [Channel; 3] = [ch("H", 0.0, 360.0, "°"), ch("S", 0.0, 100.0, "%"), ch("B", 0.0, 100.0, "%")];
const CMYK: [Channel; 4] = [ch("C", 0.0, 100.0, "%"), ch("M", 0.0, 100.0, "%"), ch("Y", 0.0, 100.0, "%"), ch("K", 0.0, 100.0, "%")];
const LAB: [Channel; 3] = [ch("L", 0.0, 100.0, ""), ch("a", -128.0, 127.0, ""), ch("b", -128.0, 127.0, "")];

/// The sliders `mode` shows (none for the Hue Cube and the Color Wheel, which draw a field).
fn channels(mode: ColorPanelMode) -> &'static [Channel] {
    match mode {
        ColorPanelMode::HueCube | ColorPanelMode::ColorWheel => &[],
        ColorPanelMode::Grayscale => &GRAY,
        ColorPanelMode::Rgb | ColorPanelMode::Web => &RGB,
        ColorPanelMode::Hsb => &HSB,
        ColorPanelMode::Cmyk => &CMYK,
        ColorPanelMode::Lab => &LAB,
    }
}

/// One web-safe step (00, 33, 66, 99, CC, FF) for an 8-bit value.
fn web_step(v: f32) -> f32 {
    ((v.clamp(0.0, 255.0) / 51.0).round() * 51.0).clamp(0.0, 255.0)
}

/// `rgb` (sRGB-encoded, 0–1) as `mode`'s slider values, in the units the sliders show. Unused
/// trailing slots are 0.
pub(crate) fn to_values(mode: ColorPanelMode, rgb: [f32; 3]) -> [f32; 4] {
    let rgb = rgb.map(|v| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 });
    match mode {
        ColorPanelMode::HueCube | ColorPanelMode::ColorWheel | ColorPanelMode::Hsb => {
            let [h, s, v] = crate::color_picker_ui::rgb_to_hsv(rgb);
            [h, s * 100.0, v * 100.0, 0.0]
        }
        ColorPanelMode::Grayscale => [(1.0 - photocraft_color::convert::rgb_to_gray(rgb)) * 100.0, 0.0, 0.0, 0.0],
        ColorPanelMode::Rgb => [rgb[0] * 255.0, rgb[1] * 255.0, rgb[2] * 255.0, 0.0],
        ColorPanelMode::Web => [web_step(rgb[0] * 255.0), web_step(rgb[1] * 255.0), web_step(rgb[2] * 255.0), 0.0],
        ColorPanelMode::Cmyk => photocraft_color::convert::rgb_to_cmyk(rgb).map(|v| v * 100.0),
        ColorPanelMode::Lab => {
            let [l, a, b] = photocraft_color::convert::srgb_to_lab(rgb);
            [l, a, b, 0.0]
        }
    }
}

/// The sRGB colour (0–1, clipped to the gamut) that `mode`'s slider `values` give.
pub(crate) fn from_values(mode: ColorPanelMode, values: [f32; 4]) -> [f32; 3] {
    let v = values.map(|x| if x.is_finite() { x } else { 0.0 });
    let rgb = match mode {
        ColorPanelMode::HueCube | ColorPanelMode::ColorWheel | ColorPanelMode::Hsb => {
            crate::color_picker_ui::hsv_to_rgb(v[0].rem_euclid(360.0), (v[1] / 100.0).clamp(0.0, 1.0), (v[2] / 100.0).clamp(0.0, 1.0))
        }
        ColorPanelMode::Grayscale => {
            let g = 1.0 - (v[0] / 100.0).clamp(0.0, 1.0);
            [g, g, g]
        }
        ColorPanelMode::Rgb => [v[0] / 255.0, v[1] / 255.0, v[2] / 255.0],
        ColorPanelMode::Web => [web_step(v[0]) / 255.0, web_step(v[1]) / 255.0, web_step(v[2]) / 255.0],
        ColorPanelMode::Cmyk => photocraft_color::convert::cmyk_to_rgb(v.map(|c| (c / 100.0).clamp(0.0, 1.0))),
        ColorPanelMode::Lab => photocraft_color::convert::lab_to_srgb([v[0].clamp(0.0, 100.0), v[1].clamp(-128.0, 127.0), v[2].clamp(-128.0, 127.0)]),
    };
    rgb.map(|c| if c.is_finite() { c.clamp(0.0, 1.0) } else { 0.0 })
}

/// The values the sliders show for `rgb`, in whole units as Photoshop shows them: the last ones
/// edited (`kept`) while they still give the same 8-bit colour, else fresh ones. A grey has no hue
/// and CMYK has many inks for one colour, so recomputing them every frame would reset what was just
/// typed (as the Color Picker keeps them).
pub(crate) fn shown_values(mode: ColorPanelMode, rgb: [f32; 3], kept: Option<[f32; 4]>) -> [f32; 4] {
    let same = |a: [f32; 3], b: [f32; 3]| crate::color_picker_ui::hex(a) == crate::color_picker_ui::hex(b);
    kept.filter(|k| same(from_values(mode, *k), rgb)).unwrap_or_else(|| to_values(mode, rgb).map(f32::round))
}

/// The track of slider `i` for `values`: that one value swept over its range, the others held.
fn track_stops(mode: ColorPanelMode, i: usize, values: [f32; 4]) -> Vec<Color32> {
    let Some(c) = channels(mode).get(i) else { return Vec::new() };
    let n = if mode == ColorPanelMode::Hsb && i == 0 { 13 } else { 9 };
    (0..n)
        .map(|k| {
            let mut v = values;
            if let Some(slot) = v.get_mut(i) {
                *slot = c.lo + (c.hi - c.lo) * k as f32 / (n - 1) as f32;
            }
            // A hue track at full saturation and brightness, as Photoshop draws it.
            if mode == ColorPanelMode::Hsb && i == 0 {
                v[1] = v[1].max(60.0);
                v[2] = v[2].max(60.0);
            }
            c32(from_values(mode, v))
        })
        .collect()
}

fn c32(rgb: [f32; 3]) -> Color32 {
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    Color32::from_rgb(q(rgb[0]), q(rgb[1]), q(rgb[2]))
}

/// The colour the panel edits: the background while its chip is framed, else the foreground.
fn edited(app: &PhotocraftApp) -> [f32; 3] {
    let c = if app.ui.color_panel.background { app.session.tools.background } else { app.session.tools.foreground };
    [c[0], c[1], c[2]]
}

/// Set the colour the panel edits.
pub(crate) fn set_edited(app: &mut PhotocraftApp, rgb: [f32; 3]) {
    let c = [rgb[0], rgb[1], rgb[2], 1.0];
    if app.ui.color_panel.background {
        app.session.tools.background = c;
    } else {
        app.session.tools.foreground = c;
        crate::type_tool::foreground_changed(app);
    }
}

/// The Color tab. `pro` is Photoshop's themes, whose Hue Cube fills the group; the other themes
/// put HSB sliders under it.
pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui, pro: bool) {
    let room = Room { fill: ui.data(|d| d.get_temp::<f32>(crate::panels::color_field_fill_id())).filter(|f| pro && f.is_finite()), top: ui.cursor().top() };
    match app.ui.color_panel.mode {
        ColorPanelMode::HueCube if pro => crate::panels::color_field(app, ui),
        ColorPanelMode::HueCube => crate::panels::color_picker(app, ui),
        ColorPanelMode::ColorWheel => wheel(app, ui, room),
        mode => sliders(app, ui, mode, room),
    }
}

/// The height the dock gives the Color tab (Photoshop's themes), so the wheel and the ramp fit the
/// group rather than scrolling in it.
#[derive(Clone, Copy, Debug)]
struct Room {
    /// The group body's height, when the dock reports it.
    fill: Option<f32>,
    /// Where the tab's content starts.
    top: f32,
}

impl Room {
    /// The height left below the cursor, when known.
    fn left(self, ui: &egui::Ui) -> Option<f32> {
        self.fill.map(|f| f - (ui.cursor().top() - self.top))
    }
}

/// The chips column of a slider or wheel mode, with Photoshop's out-of-gamut warning under it: a
/// triangle and the nearest printable colour, which a click picks.
fn chips_column(app: &mut PhotocraftApp, ui: &mut egui::Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(38.0, height.max(38.0)), Sense::hover());
    crate::panels::field_chips(app, ui, Rect::from_min_size(rect.min, vec2(38.0, 38.0)));
    let rgb = edited(app);
    let Some(fix) = crate::color_picker_ui::out_of_gamut(rgb) else { return };
    if rect.height() < 58.0 {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let warn = Rect::from_min_size(pos2(rect.left() + 3.0, rect.top() + 42.0), vec2(14.0, 14.0));
    let swatch = Rect::from_min_size(pos2(warn.right() + 3.0, warn.top()), vec2(14.0, 14.0));
    let p = ui.painter();
    p.add(egui::Shape::convex_polygon(
        vec![pos2(warn.center().x, warn.top()), warn.right_bottom(), warn.left_bottom()],
        Color32::from_rgb(240, 196, 40),
        Stroke::new(1.0, Color32::from_black_alpha(140)),
    ));
    p.text(pos2(warn.center().x, warn.bottom() - 1.0), Align2::CENTER_BOTTOM, "!", FontId::proportional(10.0), Color32::BLACK);
    p.rect_filled(swatch, 1.0, c32(fix));
    p.rect_stroke(swatch, 1.0, Stroke::new(1.0, t.field_border), StrokeKind::Outside);
    let resp = ui
        .interact(warn.union(swatch), ui.id().with("gamut-fix"), Sense::click())
        .on_hover_text(tl!("Out of gamut for printing: click to pick the nearest printable colour"));
    if resp.clicked() {
        set_edited(app, fix);
    }
}

/// One compact slider row: label, dynamic track and value field. Returns whether it was edited.
fn slider_line(ui: &mut egui::Ui, c: &Channel, value: &mut f32, stops: Option<&[Color32]>) -> bool {
    let t = Tokens::get(ui.ctx());
    let field_w = 46.0;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (lr, _) = ui.allocate_exact_size(vec2(12.0, 20.0), Sense::hover());
        ui.painter().text(lr.left_center(), Align2::LEFT_CENTER, c.label, FontId::proportional(12.0), t.text_dim);
        let slider_w = (ui.available_width() - field_w - 6.0).max(60.0);
        let s = ui.allocate_ui(vec2(slider_w, 20.0), |ui| widgets::slider_with(ui, value, c.lo..=c.hi, stops, widgets::RowGestures::default())).inner;
        let f = widgets::value_field(ui, value, c.lo..=c.hi, c.suffix, field_w);
        changed = s.changed() || f.changed();
    });
    changed
}

fn values_key(mode: ColorPanelMode, bg: bool) -> egui::Id {
    egui::Id::new(("color-panel-values", mode, bg))
}

/// A slider mode: the chips, one slider per channel and the ramp.
fn sliders(app: &mut PhotocraftApp, ui: &mut egui::Ui, mode: ColorPanelMode, room: Room) {
    let chans = channels(mode);
    let bg = app.ui.color_panel.background;
    let key = values_key(mode, bg);
    let rgb = edited(app);
    let mut values = shown_values(mode, rgb, ui.data(|d| d.get_temp(key)));
    let dynamic = app.session.prefs().interface.dynamic_color_sliders;
    let rows_h = chans.len() as f32 * 24.0;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        chips_column(app, ui, rows_h);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, c) in chans.iter().enumerate() {
                let stops = dynamic.then(|| track_stops(mode, i, values));
                if let Some(v) = values.get_mut(i) {
                    changed |= slider_line(ui, c, v, stops.as_deref());
                }
            }
        });
    });
    if changed {
        values = values.map(f32::round);
        if mode == ColorPanelMode::Web {
            for v in values.iter_mut().take(3) {
                *v = web_step(*v);
            }
        }
        set_edited(app, from_values(mode, values));
        ui.data_mut(|d| d.insert_temp(key, values));
    }
    ui.add_space(6.0);
    // The ramp takes what the group has left (12–36 pt).
    let h = room.left(ui).map_or(36.0, |left| (left - 4.0).clamp(12.0, 36.0));
    ramp(app, ui, mode, h);
}

/// The colour ramp under the sliders: Photoshop's RGB spectrum (hue across, white at the top and
/// black at the bottom), a grey ramp in Grayscale, and white and black squares at its end. A click
/// or drag picks the colour under the pointer.
fn ramp(app: &mut PhotocraftApp, ui: &mut egui::Ui, mode: ColorPanelMode, h: f32) {
    let t = Tokens::get(ui.ctx());
    let w = ui.available_width().max(60.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click_and_drag());
    let ends = Rect::from_min_max(pos2(rect.right() - 14.0, rect.top()), rect.max);
    let field = Rect::from_min_max(rect.min, pos2(ends.left() - 2.0, rect.bottom()));
    let gray = mode == ColorPanelMode::Grayscale;
    let web = mode == ColorPanelMode::Web;
    let mut mesh = egui::Mesh::default();
    let (cols, rows) = if gray { (1usize, 1usize) } else { (24usize, 2usize) };
    for j in 0..=rows {
        for i in 0..=cols {
            let (x, y) = (i as f32 / cols as f32, j as f32 / rows as f32);
            let mut c = ramp_color(gray, x, y);
            if web {
                c = c.map(|v| web_step(v * 255.0) / 255.0);
            }
            mesh.colored_vertex(pos2(field.left() + x * field.width(), field.top() + y * field.height()), c32(c));
        }
    }
    let wv = (cols + 1) as u32;
    for j in 0..rows as u32 {
        for i in 0..cols as u32 {
            let a = j * wv + i;
            mesh.add_triangle(a, a + 1, a + wv + 1);
            mesh.add_triangle(a, a + wv + 1, a + wv);
        }
    }
    ui.painter().add(mesh);
    let half = ends.height() / 2.0;
    let white = Rect::from_min_size(ends.min, vec2(ends.width(), half));
    let black = Rect::from_min_size(pos2(ends.left(), ends.top() + half), vec2(ends.width(), half));
    ui.painter().rect_filled(white, 0.0, Color32::WHITE);
    ui.painter().rect_filled(black, 0.0, Color32::BLACK);
    ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, t.separator), StrokeKind::Outside);
    if (resp.dragged() || resp.clicked())
        && let Some(p) = resp.interact_pointer_pos()
    {
        let picked = ramp_pick(field, white, black, gray, p);
        let picked = if web { picked.map(|v| web_step(v * 255.0) / 255.0) } else { picked };
        set_edited(app, picked);
    }
}

/// The ramp's colour at `x`, `y` (0–1 across and down the field).
fn ramp_color(gray: bool, x: f32, y: f32) -> [f32; 3] {
    if gray {
        let g = 1.0 - x.clamp(0.0, 1.0);
        return [g, g, g];
    }
    let hue = (1.0 - x.clamp(0.0, 1.0)) * 360.0;
    let y = y.clamp(0.0, 1.0);
    if y < 0.5 { crate::color_picker_ui::hsv_to_rgb(hue, y * 2.0, 1.0) } else { crate::color_picker_ui::hsv_to_rgb(hue, 1.0, 1.0 - (y - 0.5) * 2.0) }
}

/// The colour the ramp gives for a press at `p`.
fn ramp_pick(field: Rect, white: Rect, black: Rect, gray: bool, p: Pos2) -> [f32; 3] {
    if white.contains(p) {
        return [1.0; 3];
    }
    if black.contains(p) {
        return [0.0; 3];
    }
    let x = ((p.x - field.left()) / field.width().max(1.0)).clamp(0.0, 1.0);
    let y = ((p.y - field.top()) / field.height().max(1.0)).clamp(0.0, 1.0);
    ramp_color(gray, x, y)
}

/// Where a press at `offset` from the wheel's centre lands: hue in degrees (0° to the right,
/// counter-clockwise) and saturation 0–1 (the rim is full saturation).
pub(crate) fn wheel_pick(offset: egui::Vec2, radius: f32) -> (f32, f32) {
    if radius <= 0.0 || !offset.x.is_finite() || !offset.y.is_finite() {
        return (0.0, 0.0);
    }
    let hue = (-offset.y).atan2(offset.x).to_degrees().rem_euclid(360.0);
    let sat = (offset.length() / radius).clamp(0.0, 1.0);
    (hue, sat)
}

/// The Color Wheel: hue around the disc, saturation out from the centre, at the brightness of the
/// slider under it.
fn wheel(app: &mut PhotocraftApp, ui: &mut egui::Ui, room: Room) {
    let t = Tokens::get(ui.ctx());
    let bg = app.ui.color_panel.background;
    let key = values_key(ColorPanelMode::Hsb, bg);
    let rgb = edited(app);
    let mut values = shown_values(ColorPanelMode::Hsb, rgb, ui.data(|d| d.get_temp(key)));
    let avail = ui.available_width();
    // As wide as the panel allows, and short enough to leave the brightness slider in the group.
    let tall = room.left(ui).map_or(220.0, |left| left - 34.0);
    let size = (avail - 46.0).min(tall).clamp(56.0, 220.0);
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        chips_column(app, ui, size);
        let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
        let centre = rect.center();
        let radius = size / 2.0 - 2.0;
        let brightness = (values[2] / 100.0).clamp(0.0, 1.0);
        let (rings, segs) = (8usize, 72usize);
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(centre, c32([brightness; 3]));
        for r in 1..=rings {
            let s = r as f32 / rings as f32;
            for k in 0..segs {
                let a = k as f32 / segs as f32 * std::f32::consts::TAU;
                let pos = centre + vec2(a.cos(), -a.sin()) * radius * s;
                mesh.colored_vertex(pos, c32(crate::color_picker_ui::hsv_to_rgb(a.to_degrees(), s, brightness)));
            }
        }
        let segs32 = segs as u32;
        for k in 0..segs32 {
            mesh.add_triangle(0, 1 + k, 1 + (k + 1) % segs32);
        }
        for r in 1..rings as u32 {
            let inner = 1 + (r - 1) * segs32;
            let outer = 1 + r * segs32;
            for k in 0..segs32 {
                let k1 = (k + 1) % segs32;
                mesh.add_triangle(inner + k, outer + k, outer + k1);
                mesh.add_triangle(inner + k, outer + k1, inner + k1);
            }
        }
        ui.painter().add(mesh);
        ui.painter().circle_stroke(centre, radius, Stroke::new(1.0, t.separator));
        if (resp.dragged() || resp.clicked())
            && let Some(p) = resp.interact_pointer_pos()
        {
            let (h, s) = wheel_pick(p - centre, radius);
            values[0] = h;
            values[1] = s * 100.0;
            changed = true;
        }
        let a = values[0].to_radians();
        let knob = centre + vec2(a.cos(), -a.sin()) * radius * (values[1] / 100.0).clamp(0.0, 1.0);
        ui.painter().circle_stroke(knob, 5.0, Stroke::new(1.5, Color32::WHITE));
        ui.painter().circle_stroke(knob, 6.5, Stroke::new(1.0, Color32::from_black_alpha(160)));
    });
    ui.add_space(4.0);
    let dynamic = app.session.prefs().interface.dynamic_color_sliders;
    let stops = dynamic.then(|| track_stops(ColorPanelMode::Hsb, 2, values));
    changed |= slider_line(ui, &HSB[2], &mut values[2], stops.as_deref());
    if changed {
        values = values.map(f32::round);
        set_edited(app, from_values(ColorPanelMode::Hsb, values));
        ui.data_mut(|d| d.insert_temp(key, values));
    }
}

/// The Color panel's own entries at the top of its panel menu: the modes (the current one
/// ticked) and the two copy commands.
pub fn panel_menu(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    for (i, m) in ColorPanelMode::ALL.into_iter().enumerate() {
        // Photoshop separates the fields from the sliders.
        if i == 2 {
            ui.separator();
        }
        let prefix = if app.ui.color_panel.mode == m { "✔ " } else { "   " };
        if ui.button(format!("{prefix}{}", tl!(m.label()))).clicked() {
            app.ui.color_panel.mode = m;
            ui.close();
        }
    }
    ui.separator();
    let hex = crate::color_picker_ui::hex(edited(app)).trim_start_matches('#').to_ascii_lowercase();
    if ui.button(format!("   {}", tl!("Copy Color as HTML"))).clicked() {
        ui.ctx().copy_text(html_color(&hex));
        ui.close();
    }
    if ui.button(format!("   {}", tl!("Copy Color's Hex Code"))).clicked() {
        ui.ctx().copy_text(hex);
        ui.close();
    }
}

/// Copy Color as HTML's text, as Photoshop writes it.
pub(crate) fn html_color(hex: &str) -> String {
    format!("COLOR=\"#{hex}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tol)
    }

    #[test]
    fn every_slider_mode_round_trips_a_colour() {
        for mode in ColorPanelMode::ALL {
            for rgb in [[0.2, 0.4, 0.6], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]] {
                let back = from_values(mode, to_values(mode, rgb));
                let tol = match mode {
                    // Grey keeps only the luminance; the web sliders snap to 0x33 steps.
                    ColorPanelMode::Grayscale | ColorPanelMode::Web => continue,
                    // CMYK goes through the colour-managed press profile; colours outside its
                    // gamut (pure red) come back as the nearest printable one, as they should.
                    ColorPanelMode::Cmyk if crate::color_picker_ui::out_of_gamut(rgb).is_some() => continue,
                    ColorPanelMode::Cmyk => 0.03,
                    _ => 0.01,
                };
                assert!(close(back, rgb, tol), "{mode:?}: {rgb:?} came back as {back:?}");
            }
        }
    }

    #[test]
    fn grayscale_slider_is_percent_ink() {
        assert!(to_values(ColorPanelMode::Grayscale, [1.0; 3])[0].abs() < 1e-4, "white is 0 % ink");
        assert!((to_values(ColorPanelMode::Grayscale, [0.0; 3])[0] - 100.0).abs() < 1e-4, "black is 100 %");
        let g = from_values(ColorPanelMode::Grayscale, [25.0, 0.0, 0.0, 0.0]);
        assert!(close(g, [0.75; 3], 1e-5));
    }

    #[test]
    fn web_sliders_snap_to_web_safe_steps() {
        assert_eq!(to_values(ColorPanelMode::Web, [0.21, 0.5, 0.99]), [51.0, 153.0, 255.0, 0.0]);
        let c = from_values(ColorPanelMode::Web, [60.0, 120.0, 250.0, 0.0]);
        assert!(close(c, [0.2, 0.4, 1.0], 1e-5), "{c:?}");
    }

    #[test]
    fn kept_values_survive_while_they_give_the_same_colour() {
        // Black in HSB: any hue and saturation give it, so the typed ones are kept.
        let kept = [200.0, 80.0, 0.0, 0.0];
        assert_eq!(shown_values(ColorPanelMode::Hsb, [0.0; 3], Some(kept)), kept);
        // A different colour replaces them.
        let red = shown_values(ColorPanelMode::Hsb, [1.0, 0.0, 0.0], Some(kept));
        assert!(red[0].abs() < 0.5 && (red[1] - 100.0).abs() < 0.5 && (red[2] - 100.0).abs() < 0.5, "{red:?}");
    }

    #[test]
    fn hostile_values_give_a_valid_colour() {
        for mode in ColorPanelMode::ALL {
            for v in [[f32::NAN; 4], [f32::INFINITY; 4], [-1e9; 4], [1e9; 4]] {
                let c = from_values(mode, v);
                assert!(c.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)), "{mode:?} {v:?} → {c:?}");
            }
            let t = to_values(mode, [f32::NAN, 2.0, -1.0]);
            assert!(t.iter().all(|x| x.is_finite()), "{mode:?} → {t:?}");
        }
    }

    #[test]
    fn wheel_maps_angle_to_hue_and_radius_to_saturation() {
        let (h, s) = wheel_pick(vec2(50.0, 0.0), 100.0);
        assert!(h.abs() < 1e-3 && (s - 0.5).abs() < 1e-5, "right of centre is red at half saturation");
        let (h, s) = wheel_pick(vec2(0.0, -200.0), 100.0);
        assert!((h - 90.0).abs() < 1e-3 && (s - 1.0).abs() < 1e-5, "above is 90°, clamped to the rim");
        assert_eq!(wheel_pick(vec2(f32::NAN, 0.0), 100.0), (0.0, 0.0));
        assert_eq!(wheel_pick(vec2(1.0, 1.0), 0.0), (0.0, 0.0));
    }

    #[test]
    fn ramp_ends_are_white_and_black() {
        let field = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 36.0));
        let white = Rect::from_min_size(pos2(102.0, 0.0), vec2(14.0, 18.0));
        let black = Rect::from_min_size(pos2(102.0, 18.0), vec2(14.0, 18.0));
        assert_eq!(ramp_pick(field, white, black, false, pos2(105.0, 5.0)), [1.0; 3]);
        assert_eq!(ramp_pick(field, white, black, false, pos2(105.0, 30.0)), [0.0; 3]);
        // The middle row of the spectrum is fully saturated, the left end red.
        let red = ramp_pick(field, white, black, false, pos2(0.0, 18.0));
        assert!(close(red, [1.0, 0.0, 0.0], 1e-4), "{red:?}");
        let g = ramp_pick(field, white, black, true, pos2(50.0, 10.0));
        assert!(close(g, [0.5; 3], 1e-4), "{g:?}");
    }

    #[test]
    fn html_copy_is_photoshops_format() {
        assert_eq!(html_color("ff7a59"), "COLOR=\"#ff7a59\"");
    }

    #[test]
    fn hsb_hue_track_sweeps_the_spectrum() {
        let stops = track_stops(ColorPanelMode::Hsb, 0, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(stops.len(), 13);
        assert_ne!(stops[0], stops[4], "a black colour still shows a hue track");
        assert!(track_stops(ColorPanelMode::Rgb, 7, [0.0; 4]).is_empty(), "no such slider");
    }
}
