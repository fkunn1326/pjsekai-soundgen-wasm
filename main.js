import { createFFmpeg, fetchFile } from "@ffmpeg/ffmpeg";
// import init, {main} from "./pjsekai-soundgen-wasm/pkg/pjsekai_soundgen_wasm.js";

// ffmpeg.jsのロード
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

async function download_bgm(id){
  // 変数の宣言
  var AudioList,AudioBin
  var Audio = []
  // 曲(音源)のダウンロード
  AudioList = await (await fetch(`https://pjsekai-ui-server.vercel.app/repository/${id}/bgm.mp3`)).json()
    // 分割された曲を結合
    for(var AudioUrl of AudioList){
      AudioBin = await (await fetch(`https://pjsekai-ui-server.vercel.app${AudioUrl}`)).arrayBuffer()
      console.log(AudioBin)
      Audio.push(AudioBin)
    }
    AudioBin = new Blob(Audio, {type: "audio/mpeg"})
    ffmpeg.FS('writeFile', 'bgm.mp3', await fetchFile(AudioBin));
  // 曲の変換
  await ffmpeg.run('-i', 'bgm.mp3', '-ac', '2', '-f', 's16le', '-ar', '48k', '-loglevel', 'warning', 'output.wav');
  var data = ffmpeg.FS('readFile', 'output.wav');
  ffmpeg.exit();
  //Soundgen-wasm
  const { run } = wasm_bindgen;
  const maxWorkers = navigator.hardwareConcurrency || 4;
  await console.log(maxWorkers);
  async function wasm_run() {
    await wasm_bindgen('./pjsekai-soundgen-wasm/pkg/pjsekai_soundgen_wasm_bg.wasm');
    run(data, id, 1.0, 0.0, 100000);
  }
  wasm_run();
}

download_bgm("Uozt01KNf5WJ")