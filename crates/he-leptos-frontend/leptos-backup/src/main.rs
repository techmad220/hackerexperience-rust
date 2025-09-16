use leptos::*;
use wasm_bindgen::prelude::*;

mod api;
mod app;
mod components;  
mod pages;
mod state;
mod utils;

use app::App;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

fn main() {
    hydrate();
}