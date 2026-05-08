import init from '../pkg/mcavatar.js';
import wasmModule from '../pkg/mcavatar_bg.wasm';

export * from '../pkg/mcavatar.js';

await init({ module_or_path: wasmModule });
