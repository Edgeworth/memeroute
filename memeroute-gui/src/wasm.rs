use eframe::wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::{self};

#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = TemplateApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
