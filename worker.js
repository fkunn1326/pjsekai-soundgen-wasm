import { threads } from 'wasm-feature-detect';
import * as Comlink from 'comlink';

async function importModule() {
  if (!(await threads())) {
    return;
  }

  const thread = await import('./pjsekai-soundgen-wasm/pkg/pjsekai_soundgen_wasm.js');
  await thread.default();
  await thread.initThreadPool(navigator.hardwareConcurrency);

  return Comlink.proxy({
    main: thread.main,
    sum: thread.sum,
  });
}

Comlink.expose({
  thread: importModule(),
});