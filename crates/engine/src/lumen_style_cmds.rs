//! Lumen's quick styles (Image › Quick Styles, "Estilos rápidos" in Spanish): one-click looks built from ordinary adjustment
//! layers, so every step stays editable and the whole style undoes as one history step.

use photocraft_doc::{Layer, LayerContent};
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

pub fn specs() -> Vec<CommandSpec> {
    [style_run!(0), style_run!(1), style_run!(2), style_run!(3), style_run!(4), style_run!(5)].into_iter().filter_map(|(i, run)| spec(i, run)).collect()
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
        assert_eq!(specs().len(), STYLES.len());
        for (id, ..) in STYLES {
            assert!(crate::command_specs().iter().any(|c| c.id == *id), "{id} isn't registered");
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
}
