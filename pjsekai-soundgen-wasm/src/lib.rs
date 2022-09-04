#![feature(async_closure)]
pub mod console;
pub mod sonolus;
pub mod sound;
pub mod utils;

use core::panic;
use std::collections::HashMap;

use web_sys::console::*;
use js_sys::Array;

use console::show_title;
use sonolus::*;
use sound::Sound;
use crate::sound::{NOTE_NAME_MAP, SOUND_MAP};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

use wasm_thread as thread;

struct Args {
    bgm_override: Option<String>,
    bgm_volume: f32,
    shift: f32,
    silent: bool,
    output: Option<String>,
    id: Option<String>,
    notes_per_thread: Option<usize>,
}

fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::main;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn dummy_main() {}

    #[wasm_bindgen]
    pub async fn run(bgm_raw: Vec<u8>, id: String, bgm_volume: f32, shift: f32, notes_per_thread: i32) {
        console_log::init().unwrap();
        console_error_panic_hook::set_once();
        main(bgm_raw, id, bgm_volume, shift, notes_per_thread).await;
    }
}

// #[cfg(target_arch = "wasm32")]
// fn main() {
//     #[cfg(not(target_arch = "wasm32"))]
//     env_logger::init_from_env(
//         env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
//     );

//     let mut threads = vec![];

//     for i in 0..10 {
//         threads.push(thread::spawn  (move ||
//             log::info!("Starting thread")
//         ));
//     };

//     for t in threads {
//         t.join().unwrap();
//     }
// }

// #[cfg(target_arch = "wasm32")]
async fn main(bgm_raw: Vec<u8>, id: String, bgm_volume: f32, shift: f32, notes_per_thread: i32) {
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
    let bgm = bgm_raw * bgm_volume;
    log_3(&JsValue::from("%cInfo%c ノーツを読み込んでいます..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    let (tap_sound_timings, connect_note_timings) = level.get_sound_timings(shift).await;
    log_3(&JsValue::from("%cInfo%c ノーツのSEを読み込んでいます..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));

    let mut threads = vec![];

    let note_sound_data = SOUND_MAP
        .iter()
        .map(|(_key, value)| {
            let raw = Sound::load(&value.0.to_vec());
            (value.1, raw)
    }).collect::<HashMap<_, _>>();

    let (tx, rx) = async_channel::unbounded();

    for (note, times) in tap_sound_timings.clone() {
        log_3(&JsValue::from(&format!("%cInfo%c {:?}, {:?}", note, times)), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
        let sound = note_sound_data.get(note.as_str()).unwrap().clone();
        let is_critical = note.starts_with("critical_");

        let thread_num: usize = (times.len() as f32 / (notes_per_thread as f32)).ceil() as usize;

        log_3(&JsValue::from(&format!("%cInfo%c スレッド数: {}", thread_num)), &JsValue::from(""), &JsValue::from(""));

        for i in 0..thread_num {
            let lsound = sound.clone();
            let mut ltx = tx.clone();
            let ltimes = times[(i * (notes_per_thread as usize))..=((i + 1) * (notes_per_thread as usize)).min(times.len() - 1)].to_vec();
            threads.push(thread::spawn(move || {
                let mut local_sound = Sound::empty(None);
                for (i, time) in ltimes.iter().enumerate() {
                    if i == (notes_per_thread as usize) {
                        continue;
                    }
                    let next_time = ltimes.get(i + 1).unwrap_or(&(*time + 5.0)) + shift;
                    
                    //log_1(&JsValue::from(&format!("{}, {}, {}", type_of(&lsound), time.clone(), &next_time)));
                    
                    local_sound = local_sound.overlay_until(&lsound, time.clone(), next_time).clone();
                }
                ltx.send(local_sound);
            }))
        }
    }
    for (note, times) in connect_note_timings.clone() {
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
        let mut ltx = tx.clone();
        threads.push(thread::spawn(move || {
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
            ltx.send(local_sound);
        }));
    }

    let thread_len = threads.len();
    let draw_thread = thread::spawn(move || log_3(&JsValue::from(&format!("%cInfo%c {}スレッドで処理を開始します。", thread_len)), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from("")));
    let mut merged_sounds = Sound::empty(None);
    for _ in 0..threads.len() {
        let received = rx.recv().await.unwrap();
        merged_sounds = merged_sounds.overlay_at(&received, 0.0);
        drop(received);
    }
    log_3(&JsValue::from("%cInfo%c BGMとSEを合成中..."), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
    spawn_local(async move {
    });
    // log_3(&JsValue::from(&format!("%cInfo%c {:?}, {:?}", note, times)), &JsValue::from("color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;"), &JsValue::from(""));
}