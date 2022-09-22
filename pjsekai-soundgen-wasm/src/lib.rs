#![feature(async_closure)]
pub mod console;
pub mod sonolus;
pub mod sound;
pub mod utils;

use std::panic;
use std::collections::HashMap;

use js_sys::Uint8Array;
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
pub async fn main(bgm_raw: Vec<u8>, id: String, bgm_volume: f32, shift: f32, notes_per_thread: usize) -> Uint8Array {
    let bgm_def = &bgm_raw;
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
            let raw = Sound {
                data: value.0.to_vec().chunks_exact(2).into_iter().map(|a| i16::from_le_bytes([a[0], a[1]])).collect(),
                bitrate: 48000,
            };
            (value.1, raw)
    }).collect::<HashMap<_, _>>();

    let (tx, rx) = crossbeam_channel::unbounded();

    tap_sound_timings.clone()
                     .par_iter()
                     .for_each(|(note, times)| {
                        log_1(&JsValue::from(&format!("{:?}: {:?}", note, times)));
                        let sound = note_sound_data.get(note.as_str()).unwrap().clone();
                        (0..1).into_par_iter()
                                       .for_each(|i| {
                                            let lsound = sound.clone();
                                            let ltx = tx.clone();
                                            let ltimes = times;

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

    connect_note_timings.clone()
                        .par_iter()
                        .for_each(|(note, times)|{
                            let mut events = vec![];
                            for (start, end) in times.clone() {
                                events.push((1, start));
                                events.push((-1, end));
                            }
                            events.sort_by(|a, b| {
                                if a.1 == b.1 {
                                    b.0.partial_cmp(&a.0).unwrap()
                                } else {
                                    a.1.partial_cmp(&b.1).unwrap()
                                }
                            });
                            let sound = note_sound_data.get(note.as_str()).unwrap().clone();
                            let ltx = tx.clone();

                            let mut local_sound = Sound::empty(None);
                            let lsound = sound.clone();
                            drop(sound);
                            let mut current = 0;
                            let mut start_time = 0.0;
                            for (sign, time) in events.clone() {
                                current += sign;
                                if sign == -1 && current == 0 {
                                    local_sound = local_sound.overlay_loop(&lsound, start_time, time);
                                } else if sign == 1 && current == 1 {
                                    start_time = time;
                                }
                            }
                            assert_eq!(current, 0);
                            ltx.send(local_sound).unwrap();
                        });
                        
    let mut merged_sounds = Sound::empty(None);
    for _ in 0..rx.len() {
        let received = rx.recv().unwrap();
        merged_sounds = merged_sounds.overlay_at(&received, 0.0);
        drop(received);
    }
    let mut final_bgm: Sound;
    final_bgm = bgm;
    final_bgm = final_bgm.overlay_at(&merged_sounds, 0.0);
    let export_bgm = &final_bgm.data.iter().flat_map(|a| a.to_le_bytes().to_vec()).collect::<Vec<u8>>();

    log_3(&JsValue::from("%cInfo%c BGMとSEを合成中..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    return Uint8Array::new(&unsafe { Uint8Array::view(&export_bgm) }.into());
}