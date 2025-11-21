mod core;
mod state;
mod rendering;
mod input;
mod app;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(start)]
pub fn start() {
    spawn_local(async {
        app::run().await;
    });
}
