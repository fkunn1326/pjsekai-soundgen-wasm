use crate::utils::*;
use web_sys::console::log;
use js_sys::Array;
use wasm_bindgen::prelude::*;

pub fn show_title() {
    let array = js_sys::Array::new();
    array.push(&"
%c== pjsekai-soundgen-rust ------------------------------------------
%c  pjsekai-soundgen-rust / Rust版プロセカ風譜面音声生成ツール
%c  Developed by 名無し｡(@sevenc-nanashi)
%c  https://github.com/sevenc-nanashi/pjsekai-soundgen-rust
".into());
    array.push(&"color: #00b5c9".into());
    array.push(&"color: #00afc7".into());
    array.push(&"color: #48b0d5".into());
    array.push(&"color: #ff5a91".into());
    log(&array);
}