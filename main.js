import { createFFmpeg, fetchFile } from "@ffmpeg/ffmpeg";

let ffmpeg = null;

const transcode = async ({ target: { files } }) => {
  if (ffmpeg === null) {
    ffmpeg = createFFmpeg({ log: true });
  }
  const message = document.getElementById('message');
  const { name } = files[0];
  message.innerHTML = 'Loading ffmpeg-core.js';
  if (!ffmpeg.isLoaded()) {
    await ffmpeg.load();
  }
  ffmpeg.FS('writeFile', name, await fetchFile(files[0]));
  message.innerHTML = 'Start transcoding';
  await ffmpeg.run('-i', name,  'output.mp4');
  message.innerHTML = 'Complete transcoding';
  const data = ffmpeg.FS('readFile', 'output.mp4');

  const video = document.getElementById('output-video');
  video.src = URL.createObjectURL(new Blob([data.buffer], { type: 'video/mp4' }));
}
const elm = document.getElementById('uploader');
elm.addEventListener('change', transcode);

const cancel = () => {
  try {
    ffmpeg.exit();
  } catch(e) {}
  ffmpeg = null;
}


async function main(id){
  //変数の宣言
  var SongData,AudioList, AudioBin
  var Audio = []
  //曲データのダウンロード
  SongData = await (await fetch(`https://pjsekai-ui-server.vercel.app/levels/${id}`)).json()
  console.log(SongData)
  //曲(音源)のダウンロード
  AudioList = await (await fetch(`https://pjsekai-ui-server.vercel.app${SongData["item"]["bgm"]["url"]}`)).json()
  console.log(AudioList)
  for(var AudioUrl of AudioList){
    console.log(AudioBin = await (await fetch(`https://pjsekai-ui-server.vercel.app${AudioUrl}`)).arrayBuffer())
    Audio.push(AudioBin)
  }
  AudioBin = new Blob(Audio, {type: "audio/mpeg"})
  console.log([await AudioBin.arrayBuffer()])
  var AudioEl = document.createElement("audio")
  AudioEl.src = URL.createObjectURL(new Blob(Audio, {type: "audio/mpeg"}))
  document.getElementById("test").appendChild(AudioEl)
  console.log(URL.createObjectURL(AudioBin))
}
main("Uozt01KNf5WJ")