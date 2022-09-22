import { createFFmpeg, fetchFile } from "@ffmpeg/ffmpeg";
import * as Comlink from 'comlink';


const btn_confirm = document.getElementById(`btn_confirm`);
const id = document.getElementById("chart_id");
const box = document.getElementById("chart_box");
const feedback1 = document.getElementById("chart_feedback1");
const feedback2 = document.getElementById("chart_feedback2");
const feedback3 = document.getElementById("chart_feedback3");

async function gen_sound(id){
  let ffmpeg = null;
  if (ffmpeg === null) {
    ffmpeg = createFFmpeg();
  }
  if (!ffmpeg.isLoaded()) {
    await ffmpeg.load();
  }
  ffmpeg.setLogger(({ type, message }) => {
    if (type === "info"){
      console.log(`%c${type}%c ${message}`, 'color:white; font-weight: bold; background-color:orange; padding:2px 4px; border-radius:4px;', '');
    }else if (type === 'fferr'){
      console.log(`%c${type}%c ${message}`, 'color:white; font-weight: bold; background-color:red; padding:2px 4px; border-radius:4px;', '');
    }else if (type === 'ffout'){
      console.log(`%c${type}%c ${message}`, 'color:white; font-weight: bold; background-color:blue; padding:2px 4px; border-radius:4px;', '');
    }
  });
  function cancel(){
    try {
      ffmpeg.exit();
    } catch(e) {}
    ffmpeg = null;
  }

  const info_box = document.getElementById('info_box');

  // 変数の宣言
  var AudioList,AudioBin
  var Audio = []
  // 曲(音源)のダウンロード
  info_box.innerText = "BGMをダウンロード中..."
  AudioList = await (await fetch(`https://pjsekai-ui-server.vercel.app/repository/${id}/bgm.mp3`)).json()
    // 分割された曲を結合
    for(var AudioUrl of AudioList){
      AudioBin = await (await fetch(`https://pjsekai-ui-server.vercel.app${AudioUrl}`)).arrayBuffer()
      Audio.push(AudioBin)
    }
    AudioBin = new Blob(Audio, {type: "audio/mpeg"})
    info_box.innerText = "BGMを読み込み中..."
    ffmpeg.FS('writeFile', 'bgm.mp3', await fetchFile(AudioBin));
  // 曲の変換
  await ffmpeg.run('-i', 'bgm.mp3', '-ac', '2', '-f', 's16le', '-ar', '48k', '-loglevel', 'warning', 'output.pcm');
  var data = ffmpeg.FS('readFile', 'output.pcm');
  //Soundgen-wasm
  // const { main, initThreadPool  } = wasm_bindgen;
  // const maxWorkers = navigator.hardwareConcurrency || 4;
  // async function wasm_run() {
  //   await wasm_bindgen('./pjsekai-soundgen-wasm/pkg/pjsekai_soundgen_wasm_bg.wasm');
  //   initThreadPool(navigator.hardwareConcurrency);
  //   main(data, id, 1.0, 0.0, 100000);
  // }
  // wasm_run();
  info_box.innerText = "音声を合成中...";

  (async () => {
    const thread = await Comlink.wrap(
      new Worker(new URL('./worker.js', import.meta.url), { type: 'module' })
    ).thread;
  
    const numbers = Array.from({ length: 100 }, (_, i) => i + 1);
    var export_data = await thread.main(data, id, 1.0, 0.0, navigator.hardwareConcurrency);
    export_data =  new Blob([export_data.buffer], {type: "application/octet-stream"});
    info_box.innerText = "音声を書き出し中..."
    ffmpeg.FS('writeFile', 'input.pcm', await fetchFile(export_data));
    await ffmpeg.run('-f', 's16le', '-c:a', 'pcm_s16le', '-ar', '48k', '-ac', '2', '-i', 'input.pcm', '-b:a', '480k', '-maxrate', '480k', '-bufsize', '480k', '-minrate', '480k', 'output.mp3');
    var final_data = ffmpeg.FS('readFile', 'output.mp3');
    ffmpeg.exit();

    info_box.innerText = ""

    btn_confirm.disabled = false
    btn_confirm.innerText = "音声をダウンロードする"
    btn_confirm.addEventListener('click', () => {
      const link = document.createElement('a');
      link.download = `${id}.mp3`;
      link.href = URL.createObjectURL(new Blob([final_data.buffer], { type: 'audio/mp3' }))
      link.click();
      URL.revokeObjectURL(link.href);
    }, false);
  })();
}


async function check_id(){
  
  if (id.value == ""){
    id.classList.add("is-invalid");
    feedback2.classList.add("d-none");
    feedback1.classList.remove("d-none");
  }else{
    var chart_id = ""
    
    if (id.value.startsWith("#")){
      chart_id = id.value.substring(1);
    }else{
      chart_id = id.value
    }
    if (chart_id.startsWith("frpt-")){
      chart_id = chart_id.substring(5)
    }else if (chart_id.startsWith("sweet-potato-")){
      chart_id = chart_id.substring(13)
    }

    var chart_res = await (await fetch(`https://pjsekai-ui-server.vercel.app/levels/${chart_id}`)).status
    if (chart_res === 404){
      feedback1.classList.add("d-none");
      feedback2.classList.remove("d-none");
      id.classList.add("is-invalid");
    }else{
      id.classList.remove("is-invalid");
      id.classList.add("is-valid");
      id.readonly = true;
      id.disabled = true;
      feedback1.classList.add("d-none");
      feedback2.classList.add("d-none");
      feedback3.classList.remove("d-none");
      box.classList.remove("is-invalid");
      box.classList.add("is-valid")
      btn_confirm.disabled = true;
      btn_confirm.removeEventListener(`click`, check_id);
      gen_sound(chart_id)
    }
  }
}

btn_confirm.addEventListener(`click`, check_id);

// download_bgm("jNMAmx7rTbTh")