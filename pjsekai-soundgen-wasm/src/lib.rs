#![feature(async_closure)]
pub mod console;
pub mod sonolus;
pub mod sound;
pub mod utils;

use std::panic;
use std::collections::HashMap;

use web_sys::console::*;
use js_sys::Array;

use console::show_title;
use sonolus::*;
use sound::Sound;
use crate::sound::{NOTE_NAME_MAP, SOUND_MAP};

use wasm_bindgen::{prelude::*, Clamped};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

pub use wasm_bindgen_rayon::init_thread_pool;
use rayon::prelude::*;

fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}

#[wasm_bindgen]
pub fn sum(numbers: &[i32]) -> i32 {
    numbers
    .par_iter()
    .map(|x| x * x)
    .sum()
}

#[wasm_bindgen]
pub async fn main(bgm_raw: Vec<u8>, id: String, bgm_volume: f32, shift: f32, notes_per_thread: usize) {
    console_error_panic_hook::set_once();
    show_title();
    let level = Level::fetch(&id).await.unwrap_or_else(|err| match err {
        LevelError::NotFound => {
            log_3(&JsValue::from("%cError%c 譜面が見つかりませんでした。"), &JsValue::from("color:white; font-weight: bold; background-color:red; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
            std::process::exit(1);
        }
        LevelError::InvalidFormat => {
            log_3(&JsValue::from("%cError%c サーバーが不正なデータを返しています。"), &JsValue::from("color:white; font-weight: bold; background-color:red; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
            std::process::exit(1);
        }
    });
    log_3(&JsValue::from(&format!("%cInfo%c {}を選択しました。", level)), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    log_3(&JsValue::from("%cInfo%c BGMを読み込んでいます..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    let bgm_raw = Sound::load(&bgm_raw);
    log_1(&bgm_raw.data.len().into());
    let bgm = bgm_raw * bgm_volume;
    log_3(&JsValue::from("%cInfo%c ノーツを読み込んでいます..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    let (tap_sound_timings, connect_note_timings) = level.get_sound_timings(shift).await;
    log_3(&JsValue::from("%cInfo%c ノーツのSEを読み込んでいます..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));

    let note_sound_data = SOUND_MAP
        .par_iter()
        .map(|(_key, value)| {
            let raw = Sound::load(&value.0.to_vec());
            (value.1, raw)
    }).collect::<HashMap<_, _>>();

    let (tx, rx) = crossbeam_channel::unbounded();

    tap_sound_timings.clone()
                     .par_iter()
                     .for_each(|(note, times)| {
                        log_1(&JsValue::from(&format!("{:?}: {:?}", note, times)));
                        let sound = note_sound_data.get(note.as_str()).unwrap().clone();
                        let is_critical = note.starts_with("critical_");
                        let thread_num: usize = (times.len() as f32 / (notes_per_thread as f32)).ceil() as usize;
                        (0..thread_num as usize).into_par_iter()
                                       .for_each(|i| {
                                            let lsound = sound.clone();
                                            let ltx = tx.clone();
                                            let ltimes = times[(i * notes_per_thread)..=((i + 1) * notes_per_thread).min(times.len() - 1)].to_vec();
                                            let mut local_sound = Sound::empty(None);
                                            for (i, time) in ltimes.iter().enumerate() {
                                                if i == notes_per_thread {
                                                    continue;
                                                }
                                                let next_time = ltimes.get(i + 1).unwrap_or(&(*time + 5.0)) + shift;
                                                local_sound = local_sound.overlay_until(&lsound, time.clone(), next_time);
                                            }
                                            ltx.send(local_sound).unwrap();
                                        });
                     });

    log_3(&JsValue::from("%cInfo%c BGMとSEを合成中..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    // log_3(&JsValue::from(&format!("%cInfo%c {:?}, {:?}", note, times)), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
}