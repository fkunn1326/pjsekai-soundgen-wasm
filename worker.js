import { threads } from 'wasm-feature-detect';
import * as Comlink from 'comlink';

async function importModule() {
  if (!(await threads())) {
    return;
  }

  const thread = await import('./public/wasm/pjsekai_soundgen_wasm.js');
  await thread.default();
  await thread.initThreadPool(navigator.hardwareConcurrency);

  return Comlink.proxy({
    main: thread.main,
  });
}

Comlink.expose({
  thread: importModule(),
});