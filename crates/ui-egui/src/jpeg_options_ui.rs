//! JPEG Options: the prompt File › Save As and Save a Copy show before writing a JPEG, as
//! Photoshop's does: Quality on its 0–12 scale with the Low, Medium, High and Maximum presets, and
//! Format Options (Baseline ("Standard"), Baseline Optimized, or Progressive in 3, 4 or 5 scans).
//!
//! The save is parked in [`Prompt`] while the modal is up (egui is immediate-mode) and written once
//! answered; the answer is recorded with the step (`quality`, `jpegFormat`, `scans`), so an action
//! replays it without asking. Cancel drops the save. The last answer is offered next time.

use serde_json::{Value, json};

use crate::{ExportSettings, PhotocraftApp};

/// JPEG Options › Format Options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JpegFormat {
    Baseline,
    Optimized,
    Progressive,
}

impl JpegFormat {
    /// The `jpegFormat` param name.
    pub fn key(self) -> &'static str {
        match self {
            JpegFormat::Baseline => "baseline",
            JpegFormat::Optimized => "optimized",
            JpegFormat::Progressive => "progressive",
        }
    }
}

/// An answer to JPEG Options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    /// Photoshop's 0–12 quality.
    pub quality: u8,
    pub format: JpegFormat,
    /// Progressive scans: 3, 4 or 5.
    pub scans: u8,
}

impl Default for Choice {
    fn default() -> Self {
        Choice { quality: 10, format: JpegFormat::Baseline, scans: 3 }
    }
}

/// Photoshop's quality presets and the quality each one sets.
pub const PRESETS: [(&str, u8); 4] = [("Low", 3), ("Medium", 5), ("High", 8), ("Maximum", 10)];

/// The preset a quality falls in (Low 0–4, Medium 5–7, High 8–9, Maximum 10–12).
pub fn preset_of(quality: u8) -> &'static str {
    match quality {
        0..=4 => "Low",
        5..=7 => "Medium",
        8..=9 => "High",
        _ => "Maximum",
    }
}

impl Choice {
    /// The encoder's 1–100 quality for this 0–12 one (the engine's `file.saveAs` mapping).
    pub fn encoder_quality(self) -> u8 {
        (f64::from(self.quality.min(12)) / 12.0 * 99.0 + 1.0).round() as u8
    }

    /// The nearest 0–12 quality to an encoder quality (a JPEG opened at quality `q`).
    pub fn from_encoder_quality(q: u8) -> u8 {
        ((f64::from(q.clamp(1, 100)) - 1.0) / 99.0 * 12.0).round() as u8
    }

    /// `base` with this answer's JPEG settings.
    pub fn settings(self, base: ExportSettings) -> ExportSettings {
        ExportSettings {
            jpeg_quality: Some(self.encoder_quality()),
            jpeg_progressive_scans: (self.format == JpegFormat::Progressive).then_some(self.scans.clamp(3, 5)),
            jpeg_optimize: self.format != JpegFormat::Baseline,
            ..base
        }
    }

    /// The recorded step's params for a save to `path`.
    pub fn step(self, path: &str) -> Value {
        let mut v = json!({"path": path, "quality": self.quality.min(12), "jpegFormat": self.format.key()});
        if self.format == JpegFormat::Progressive {
            v["scans"] = json!(self.scans.clamp(3, 5));
        }
        v
    }

    /// The answer a recorded step or a script gives (`quality` 0–12, `jpegFormat`, `scans`), or
    /// `None` when it gives none and the prompt would ask.
    pub fn from_params(p: &Value, cmd: &str) -> Result<Option<Choice>, String> {
        let has = |k: &str| p.get(k).is_some_and(|v| !v.is_null());
        if !has("quality") && !has("jpegFormat") {
            return Ok(None);
        }
        let quality = match p.get("quality").filter(|v| !v.is_null()) {
            None => Choice::default().quality,
            Some(v) => match v.as_f64() {
                Some(q) if q.is_finite() && (0.0..=12.0).contains(&q) => q.round() as u8,
                _ => return Err(format!("{cmd}: `quality` must be 0..12, got {v}")),
            },
        };
        let (scans, optimize) = photocraft_engine::file_cmds::jpeg_format_param(p, cmd).map_err(|e| e.to_string())?;
        let format = match (scans, optimize) {
            (Some(_), _) => JpegFormat::Progressive,
            (None, true) => JpegFormat::Optimized,
            (None, false) => JpegFormat::Baseline,
        };
        Ok(Some(Choice { quality, format, scans: scans.unwrap_or(3) }))
    }
}

/// A save waiting on the JPEG Options answer.
pub struct Prompt {
    /// The document that requested the save, even if tabs change while this is open.
    doc: photocraft_doc::DocId,
    /// Where the document goes.
    pub path: String,
    pub choice: Choice,
    /// Save a Copy, or a Save As that writes a copy: the document keeps its path and unsaved state.
    pub copy: bool,
    /// The step the answered save is recorded as (`file.saveAs` or `file.saveACopy`).
    pub record: &'static str,
}

/// `true` when `path` names a JPEG (`.jpg`, `.jpeg`, `.jpe`…).
pub fn is_jpeg_path(path: &str) -> bool {
    photocraft_engine::file_cmds::extension(path).and_then(|e| photocraft_codecs::from_extension(&e)) == Some(photocraft_codecs::Format::Jpeg)
}

/// Whether saving the active document to `path` asks for JPEG Options.
pub fn wants_prompt(app: &PhotocraftApp, path: &str) -> bool {
    is_jpeg_path(path) && app.session.active().is_some()
}

/// What the prompt starts at: the last answer, else the quality the document's JPEG was opened
/// at, else Maximum (10), baseline.
pub fn initial(app: &PhotocraftApp) -> Choice {
    if let Some(c) = app.jpeg_last {
        return c;
    }
    let opened = app.session.active().and_then(|st| st.doc.file_encoding.jpeg_quality);
    match opened {
        Some(q) => Choice { quality: Choice::from_encoder_quality(q), ..Choice::default() },
        None => Choice::default(),
    }
}

/// Parks the save behind the prompt. A prompt already waiting is never replaced.
pub fn park(app: &mut PhotocraftApp, path: String, copy: bool, record: &'static str) -> Result<(), String> {
    if app.save_options_open() {
        return Err("Answer the TIFF, Targa or JPEG Options before starting another save".into());
    }
    let doc = app.active_doc_id()?;
    let choice = initial(app);
    app.jpeg_options = Some(Prompt { doc, path, choice, copy, record });
    Ok(())
}

/// The modal's controls for `c`. Returns whether anything changed.
fn controls(ui: &mut egui::Ui, c: &mut Choice) -> bool {
    let before = *c;
    crate::widgets::section_label(ui, tl!("Image Options"));
    ui.horizontal(|ui| {
        ui.label(tl!("Quality:"));
        let mut q = f32::from(c.quality);
        if crate::widgets::value_field(ui, &mut q, 0.0..=12.0, "", 48.0).changed() {
            c.quality = q.round().clamp(0.0, 12.0) as u8;
        }
        let mut preset = preset_of(c.quality).to_string();
        let opts: Vec<(String, &str)> = PRESETS.iter().map(|(n, _)| (n.to_string(), *n)).collect();
        if crate::widgets::dropdown(ui, "jpeg-quality-preset", &mut preset, &opts, 110.0)
            && let Some((_, q)) = PRESETS.iter().find(|(n, _)| *n == preset)
        {
            c.quality = *q;
        }
    });
    let mut q = f32::from(c.quality);
    // Across the dialog, between its "smaller file" and "larger file" ends.
    ui.spacing_mut().slider_width = ui.available_width();
    if ui.add(egui::Slider::new(&mut q, 0.0..=12.0).step_by(1.0).show_value(false)).changed() {
        c.quality = q.round().clamp(0.0, 12.0) as u8;
    }
    ui.horizontal(|ui| {
        let t = crate::theme::Tokens::get(ui.ctx());
        ui.label(egui::RichText::new(tl!("smaller file")).small().color(t.text_faint));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(tl!("larger file")).small().color(t.text_faint));
        });
    });
    ui.add_space(8.0);
    crate::widgets::section_label(ui, tl!("Format Options"));
    ui.radio_value(&mut c.format, JpegFormat::Baseline, tl!("Baseline (\"Standard\")"));
    ui.radio_value(&mut c.format, JpegFormat::Optimized, tl!("Baseline Optimized"));
    ui.horizontal(|ui| {
        ui.radio_value(&mut c.format, JpegFormat::Progressive, tl!("Progressive"));
        ui.add_enabled_ui(c.format == JpegFormat::Progressive, |ui| {
            ui.label(tl!("Scans:"));
            let opts: Vec<(u8, &str)> = vec![(3, "3"), (4, "4"), (5, "5")];
            crate::widgets::dropdown(ui, "jpeg-scans", &mut c.scans, &opts, 56.0);
        });
    });
    *c != before
}

pub fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(p) = &mut app.jpeg_options else { return };
    let (mut ok, mut cancel) = (false, false);
    let mut choice = p.choice;
    let modal = egui::Modal::new(egui::Id::new("jpeg-options")).show(ctx, |ui| {
        ui.set_max_width(360.0);
        ui.set_min_width(320.0);
        ui.label(egui::RichText::new(tl!("JPEG Options")).font(crate::theme::semibold(15.0)));
        ui.add_space(4.0);
        crate::widgets::hairline(ui);
        ui.add_space(8.0);
        controls(ui, &mut choice);
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ok = crate::widgets::primary_button(ui, tl!("OK"), 84.0).clicked();
            cancel = crate::widgets::secondary_button(ui, tl!("Cancel"), 84.0).clicked();
        });
    });
    cancel |= modal.should_close();
    p.choice = choice;
    if ok || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
        confirm(app, ctx);
    } else if cancel {
        app.jpeg_options = None;
        app.ui.status = tl!("Save cancelled").into();
        app.ui.status_error = false;
    }
}

/// OK: write the parked save with the prompt's answer, record it, and remember the answer.
pub(crate) fn confirm(app: &mut PhotocraftApp, ctx: &egui::Context) {
    {
        let Some(p) = app.jpeg_options.take() else { return };
        app.jpeg_last = Some(p.choice);
        let settings = p.choice.settings(app.save_settings());
        let step = p.choice.step(&p.path);
        let written = app.with_document(p.doc, |app| app.write_document(p.path, &settings, p.copy));
        if written.is_ok() {
            // An action being recorded keeps the save with this answer (#2032).
            app.session.journal.push((p.record.into(), step));
        }
        match written {
            Err(e) => {
                app.ui.status = crate::i18n::fmt(tl!("Save failed: {error}"), &[("error", &e)]);
                app.ui.status_error = true;
            }
            Ok(Some(_)) if !p.copy => crate::discard_ui::saved_document(app, ctx, p.doc),
            // A background save releases the close prompt when it finishes (`jobs_ui`).
            Ok(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_names() {
        assert!(is_jpeg_path("a.jpg"));
        assert!(is_jpeg_path("dir/sub/a.JPEG"));
        assert!(is_jpeg_path("C:\\x\\y.jpg"));
        assert!(!is_jpeg_path("a.jpg.psd"));
        assert!(!is_jpeg_path("a.png"));
        assert!(!is_jpeg_path("jpg"));
    }

    #[test]
    fn quality_scales_match_the_engine() {
        assert_eq!(Choice { quality: 0, ..Choice::default() }.encoder_quality(), 1);
        assert_eq!(Choice { quality: 12, ..Choice::default() }.encoder_quality(), 100);
        assert_eq!(Choice { quality: 99, ..Choice::default() }.encoder_quality(), 100, "clamped");
        for q in 0..=12u8 {
            let c = Choice { quality: q, ..Choice::default() };
            assert_eq!(Choice::from_encoder_quality(c.encoder_quality()), q, "{q} round-trips");
        }
    }

    #[test]
    fn presets_cover_the_scale() {
        assert_eq!(preset_of(0), "Low");
        assert_eq!(preset_of(4), "Low");
        assert_eq!(preset_of(5), "Medium");
        assert_eq!(preset_of(8), "High");
        assert_eq!(preset_of(10), "Maximum");
        assert_eq!(preset_of(12), "Maximum");
        for (name, q) in PRESETS {
            assert_eq!(preset_of(q), name);
        }
    }

    #[test]
    fn settings_and_steps_follow_the_choice() {
        let base = ExportSettings::default();
        let c = Choice { quality: 8, format: JpegFormat::Progressive, scans: 5 };
        let s = c.settings(base.clone());
        assert_eq!(s.jpeg_quality, Some(67));
        assert_eq!(s.jpeg_progressive_scans, Some(5));
        assert!(s.jpeg_optimize);
        assert_eq!(c.step("a.jpg"), json!({"path": "a.jpg", "quality": 8, "jpegFormat": "progressive", "scans": 5}));
        let b = Choice { quality: 3, format: JpegFormat::Baseline, scans: 3 };
        let s = b.settings(base);
        assert_eq!(s.jpeg_progressive_scans, None);
        assert!(!s.jpeg_optimize);
        assert_eq!(b.step("a.jpg"), json!({"path": "a.jpg", "quality": 3, "jpegFormat": "baseline"}));
    }

    #[test]
    fn recorded_steps_parse_back() {
        let c = Choice { quality: 6, format: JpegFormat::Optimized, scans: 3 };
        assert_eq!(Choice::from_params(&c.step("x.jpg"), "file.saveAs"), Ok(Some(c)));
        let p = Choice { quality: 12, format: JpegFormat::Progressive, scans: 4 };
        assert_eq!(Choice::from_params(&p.step("x.jpg"), "file.saveAs"), Ok(Some(p)));
        assert_eq!(Choice::from_params(&json!({"path": "x.jpg"}), "file.saveAs"), Ok(None), "nothing given: the prompt asks");
        for bad in [
            json!({"quality": 13}),
            json!({"quality": -1}),
            json!({"quality": "high"}),
            json!({"jpegFormat": "lossless"}),
            json!({"jpegFormat": "progressive", "scans": 9}),
            json!({"jpegFormat": 3}),
        ] {
            assert!(Choice::from_params(&bad, "file.saveAs").is_err(), "{bad}");
        }
    }
}
