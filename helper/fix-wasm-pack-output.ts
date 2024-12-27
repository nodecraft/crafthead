import fs from 'node:fs/promises';

// strip out node stuff from wasm-pack output
let wasmJS = await fs.readFile('pkg/mcavatar.js', 'utf8');

// prepend import
if (!wasmJS.startsWith('import wasmModule from \'./mcavatar_bg.wasm\';')) {
	wasmJS = `import wasmModule from './mcavatar_bg.wasm';\n${wasmJS}`;
}

// remove a bunch of node stuff we don't need
function removeLine(line: string) {
	const lines = wasmJS.split(/\r?\n/);
	const index = lines.indexOf(line);
	if (index !== -1) {
		lines.splice(index, 1);
	} else {
		console.log(`Line not found: ${line}`);
	}
	return lines.join('\n');
}

wasmJS = removeLine('const { TextDecoder, TextEncoder } = require(`util`);');
wasmJS = removeLine('const path = require(\'path\').join(__dirname, \'mcavatar_bg.wasm\');');
wasmJS = removeLine('const bytes = require(\'fs\').readFileSync(path);');
wasmJS = removeLine('const wasmModule = new WebAssembly.Module(bytes);');

// write back
await fs.writeFile('pkg/mcavatar.js', wasmJS);
