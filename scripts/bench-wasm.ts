// Benchmarks the built WASM renderers (Minecraft and Hytale) outside of workerd, where timers are usable.
// Hytale scenarios come from the manifest written by the `render_bench` Rust example, so the
// same skins can be compared between native and WASM, and between two WASM builds.
//
// Usage: node scripts/bench-wasm.ts <out_dir> [--pkg=pkg] [--manifest=<dir>/manifest.json] [--iterations=15]

import * as fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

interface HytaleScenario {
	name: string;
	baseTexture: string;
	skin: string;
	assets: string[];
}

interface WasmExports {
	initSync(options: { module: BufferSource; }): unknown;
	get_rendered_image(skin: Uint8Array, size: number, what: string, armored: boolean, slim: boolean, game: string): Uint8Array;
	load_hytale_context(modelJson: string, animationJson: string, assetPaths: string[], assetBytes: Uint8Array[]): void;
	render_hytale_3d(
		textureBytes: Uint8Array,
		skinConfigJson: string,
		assetPaths: string[],
		assetBytes: Uint8Array[],
		viewType: string,
		size: number,
	): Uint8Array;
}

const HYTALE_ASSETS = 'assets/hytale';
const SIZES = [64, 180, 300];
const MINECRAFT_KINDS = ['avatar', 'helm', 'cube', 'body', 'bust'];
const HYTALE_VIEWS = ['avatar', 'cube', 'bust', 'body'];
const WARMUP_ITERATIONS = 3;

function readOption(name: string, fallback: string): string {
	const prefix = `--${name}=`;
	const arg = process.argv.find(entry => entry.startsWith(prefix));
	if (arg) {
		return arg.slice(prefix.length);
	}
	return fallback;
}

function median(samples: number[]): number {
	const sorted = samples.toSorted((left, right) => left - right);
	return sorted[Math.floor(sorted.length / 2)];
}

function measure(iterations: number, render: () => Uint8Array): { ms: number; output: Uint8Array; } {
	let output = render();
	for (let i = 1; i < WARMUP_ITERATIONS; i++) {
		output = render();
	}
	const samples: number[] = [];
	for (let i = 0; i < iterations; i++) {
		const started = performance.now();
		output = render();
		samples.push(performance.now() - started);
	}
	return { ms: median(samples), output };
}

// Provider paths are `assets/Common/<file>`; game files sit under Common/ on disk, bundled JSON beside it.
async function readProviderAsset(providerPath: string): Promise<Uint8Array> {
	const relative = providerPath.replace(/^assets\/(Common\/)?/, '');
	const bytes = await fs.readFile(path.join(HYTALE_ASSETS, 'Common', relative))
		.catch(() => fs.readFile(path.join(HYTALE_ASSETS, relative)));
	if (providerPath.endsWith('.json')) {
		// The worker bundles these as parsed JSON and re-serialises them, so match that minified form.
		return new TextEncoder().encode(JSON.stringify(JSON.parse(new TextDecoder().decode(bytes))));
	}
	return bytes;
}

async function main(): Promise<void> {
	const outDir = process.argv[2];
	if (!outDir || outDir.startsWith('--')) {
		throw new Error('Usage: node scripts/bench-wasm.ts <out_dir> [--pkg=pkg] [--manifest=path] [--iterations=15]');
	}
	const pkgDir = readOption('pkg', 'pkg');
	const manifestPath = readOption('manifest', '');
	const iterations = Number(readOption('iterations', '15'));
	await fs.mkdir(outDir, { recursive: true });

	const wasm = await import(pathToFileURL(path.resolve(pkgDir, 'mcavatar.js')).href) as WasmExports;
	wasm.initSync({ module: await fs.readFile(path.join(pkgDir, 'mcavatar_bg.wasm')) });

	const results: Record<string, number> = {};
	const record = async (name: string, render: () => Uint8Array) => {
		const { ms, output } = measure(iterations, render);
		results[name] = ms;
		await fs.writeFile(path.join(outDir, `${name}.png`), output);
		console.log(`${name.padEnd(40)} ${ms.toFixed(3).padStart(9)} ms`);
	};

	for (const skinName of ['steve', 'alex']) {
		const skin = await fs.readFile(`src/worker/assets/${skinName}.png`);
		for (const kind of MINECRAFT_KINDS) {
			for (const size of SIZES) {
				for (const armored of [false, true]) {
					const name = `minecraft-${skinName}-${kind}-${armored ? 'armor' : 'plain'}-${size}`;
					await record(name, () => wasm.get_rendered_image(skin, size, kind, armored, skinName === 'alex', 'minecraft'));
				}
			}
		}
	}

	if (manifestPath) {
		const decoder = new TextDecoder();
		const scenarios = JSON.parse(decoder.decode(await fs.readFile(manifestPath))) as HytaleScenario[];
		const modelJson = decoder.decode(await readProviderAsset('assets/Common/Characters/Player.blockymodel'));
		const animationJson = decoder.decode(await readProviderAsset('assets/Common/Characters/Animations/Default/Idle.blockyanim'));

		// The JSON the worker bundles and hands over once per isolate.
		const cosmeticFiles = await fs.readdir(path.join(HYTALE_ASSETS, 'Cosmetics/CharacterCreator'));
		const sharedPaths = [
			...cosmeticFiles.map(file => `assets/Common/Cosmetics/CharacterCreator/${file}`),
			'assets/Common/Particles/CosmeticParticles.json',
		];
		const sharedBytes = await Promise.all(sharedPaths.map(assetPath => readProviderAsset(assetPath)));
		const started = performance.now();
		wasm.load_hytale_context(modelJson, animationJson, sharedPaths, sharedBytes);
		console.log(`hytale context load (once per isolate): ${(performance.now() - started).toFixed(3)} ms`);

		for (const scenario of scenarios) {
			const texture = await readProviderAsset(`assets/Common/${scenario.baseTexture}`);
			const assetBytes = await Promise.all(scenario.assets.map(assetPath => readProviderAsset(assetPath)));
			for (const view of HYTALE_VIEWS) {
				for (const size of SIZES) {
					const name = `hytale-${scenario.name}-${view}-${size}`;
					await record(name, () => wasm.render_hytale_3d(
						texture,
						scenario.skin,
						scenario.assets,
						assetBytes,
						view,
						size,
					));
				}
			}
		}
	}

	const total = Object.values(results).reduce((sum, ms) => sum + ms, 0);
	console.log(`sum of medians: ${total.toFixed(3)} ms`);
	await fs.writeFile(path.join(outDir, 'timings.json'), JSON.stringify(results, null, '\t'));
}

await main();
