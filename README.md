# pjsekai-soundgen-wasm

SweetPotatoの譜面から音声を生成するツール、pjsekai-soundgenのWasm版。

>**Warning**
>このプロジェクトは動作しますが、とても不安定です。


## `cd pjsekai-soundgen-wasm && wasm-pack build --out-dir ../public/wasm -t web -- --all-features` 
rustのビルド

## `yarn dev` 
開発用サーバーを建てられます（開発用サーバーではChromeのみ使用できます。）
