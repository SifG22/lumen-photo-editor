//! The brand mark at the left of the title bar: the app icon (Lumen's camera aperture,
//! `assets/app-icon/`, see its README), where Photoshop shows its "Ps" tile. The PNG carries the
//! icon's rounded corners; it is decoded once per context into a mipmapped texture, so it stays
//! crisp at the title bar's size on any display scale.

use egui::{Color32, Context, Id, Rect, TextureHandle, TextureOptions, Ui, pos2};
use photocraft_codecs::{ChannelLayout, SampleType};

/// The product name people see: this is Lumen for PC, built on PhotoCraft. Crates, binaries,
/// file names and ids keep their upstream `photocraft` names, so upstream merges stay clean.
pub const PRODUCT_NAME: &str = "Lumen Photo Editor";

/// The upstream project Lumen is built on (credited in About › Licencias).
pub const UPSTREAM_NAME: &str = "PhotoCraft";

/// Who makes Lumen for PC.
pub const AUTHOR: &str = "SifG22";

/// Lumen's own version (About, the `.exe` details, the download names). Kept apart from the
/// workspace version, which follows upstream.
pub const VERSION: &str = "0.0.3";

/// Lumen's website (Help menu, About, start screen).
pub const WEBSITE: &str = "https://sifg.net";

/// The upstream repository Lumen is built on, credited in Help and About.
pub const UPSTREAM_REPO: &str = "https://github.com/storytold/photocraft";

/// `0.0.1`, plus the commit and date when the build recorded them: `0.0.1 (3f2a9c1d0, 2026-10-15)`.
pub fn long_version() -> String {
    use photocraft_engine::build_info::{build_date, short_sha};
    match (short_sha(), build_date()) {
        (Some(s), Some(d)) => format!("{VERSION} ({s}, {d})"),
        (Some(s), None) => format!("{VERSION} ({s})"),
        (None, Some(d)) => format!("{VERSION} ({d})"),
        (None, None) => VERSION.to_string(),
    }
}

/// A centred row of links (label, url), opened in the browser through the platform service.
pub fn link_row(app: &mut crate::PhotocraftApp, ui: &mut Ui, links: &[(&str, &str)]) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new("·").color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(*url);
            }
        }
    });
    if let Some(url) = clicked {
        crate::links::open(app, ui.ctx(), url);
    }
}

/// Menu items Lumen leaves out: upstream's community links (Help) and the agent-control
/// integrations page (Preferences). The commands stay, so upstream code and tests keep working.
pub const HIDDEN_MENU_ITEMS: &[&str] =
    &["help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue", "edit.preferences.integrations"];

/// The licence notice Lumen must ship with (PhotoCraft is MIT OR Apache-2.0; Lumen uses it under MIT),
/// plus the third-party material it bundles. Shown in About › Licencias.
/// SifG22's terms and conditions of use (About › Términos, and the installer's licence page).
pub const TERMS_TEXT: &str = include_str!("../../../assets/legal/TERMINOS.txt");

/// The licence notices, as above.
pub const LICENSE_TEXT: &str = concat!(include_str!("../../../LICENSE-MIT"), "\n\n", include_str!("../../../NOTICE"));

/// `s` with the upstream product name replaced by [`PRODUCT_NAME`]. The translation catalogs pass
/// every UI string through it (`i18n`), so upstream's strings show Lumen without being edited.
/// Text that must name PhotoCraft puts it in through a `{placeholder}`, filled after translation.
pub fn rebrand(s: &str) -> String {
    s.replace(UPSTREAM_NAME, PRODUCT_NAME)
}

/// 128 px: sharp at 20 pt on a 3× display, and the mipmaps keep it clean at 1×.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/128x128/apps/ai.storyteller.photocraft.png");

/// The icon's pixels (one transparent pixel if it couldn't be decoded, which a test rules out).
fn decode() -> egui::ColorImage {
    let rgba = photocraft_codecs::decode(ICON_PNG).ok().map(|img| img.convert(ChannelLayout::Rgba, SampleType::U8));
    match rgba {
        Some(img) if img.data().len() == img.width() as usize * img.height() as usize * 4 => {
            egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.data())
        }
        _ => egui::ColorImage::filled([1, 1], Color32::TRANSPARENT),
    }
}

/// The icon texture, uploaded on first use and kept in the context.
fn texture(ctx: &Context) -> TextureHandle {
    let id = Id::new("photocraft-brand-mark");
    if let Some(tex) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return tex;
    }
    let options = TextureOptions { mipmap_mode: Some(egui::TextureFilter::Linear), ..TextureOptions::LINEAR };
    let tex = ctx.load_texture("photocraft-brand-mark", decode(), options);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// Where the mark was painted this frame (tests and screenshots).
fn rect_id() -> Id {
    Id::new("photocraft-brand-mark-rect")
}

/// Paint the mark into `r` (square).
pub fn paint_mark(ui: &Ui, r: Rect) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(texture(ui.ctx()).id(), r, uv, Color32::WHITE);
    ui.ctx().data_mut(|d| d.insert_temp(rect_id(), r));
}

/// The rect the mark was last painted into.
#[cfg(test)]
pub fn mark_rect(ctx: &Context) -> Option<Rect> {
    ctx.data(|d| d.get_temp(rect_id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Lang, tr, tr_id, trn};

    #[test]
    fn ui_strings_name_lumen_in_every_language() {
        let es = Lang::from_code("es").unwrap();
        assert_eq!(tr(Lang::EN, "About PhotoCraft"), "About Lumen Photo Editor");
        assert_eq!(tr(es, "About PhotoCraft"), "Acerca de Lumen Photo Editor");
        assert_eq!(tr_id(Lang::EN, "help.about", "About PhotoCraft"), "About Lumen Photo Editor");
        assert_eq!(trn(Lang::EN, 2, "{n} step has no PhotoCraft equivalent yet", "{n} steps have no PhotoCraft equivalent yet"), "2 steps have no Lumen Photo Editor equivalent yet");
        // Strings that don't name the product are untouched, and the name can still be filled in.
        assert_eq!(tr(Lang::EN, "Layer"), "Layer");
        assert_eq!(rebrand("Lumen is built on PhotoCraft"), "Lumen is built on Lumen Photo Editor");
        assert_eq!(crate::i18n::fmt(tr(es, "Version {version}"), &[("version", UPSTREAM_NAME)]), "Versión PhotoCraft");
    }

    #[test]
    fn the_icon_decodes_to_a_rounded_128_px_tile() {
        let img = decode();
        assert_eq!(img.size, [128, 128]);
        // Rounded corners: the corner pixel is transparent, the centre opaque.
        assert_eq!(img.pixels[0].a(), 0);
        assert_eq!(img.pixels[64 * 128 + 64].a(), 255);
    }

    #[test]
    fn the_title_bar_paints_the_icon_from_one_cached_texture() {
        let ctx = Context::default();
        crate::PhotocraftApp::setup_context(&ctx, crate::theme::ThemeKind::ALL[0]);
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        // The system title bar hides the in-app mark (the OS draws its own icon), so paint the
        // app-drawn bar here.
        app.custom_titlebar = true;
        let mut frame = || {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| crate::panels::title_bar(&mut app, ui));
            let uploads = out.textures_delta.set.values().flat_map(|d| d.iter()).filter(|d| d.image.size() == [128, 128]).count();
            out.textures_delta.clear();
            uploads
        };
        assert_eq!(frame(), 1, "uploaded on first use");
        assert_eq!(frame(), 0, "then reused");
        assert!(mark_rect(&ctx).is_some());
    }
}
