// The purpose of this script is to extract needed assets from the Hytale Assets.zip
// The user should place their zip in the assets/ folder.
// This script should then extract the needed assets to the assets/hytale/ folder.
// We should only ever stream the assets out of the zip, never store them in memory so we don't run out of memory.

import { createWriteStream } from 'node:fs';
import * as fs from 'node:fs/promises';
import path from 'node:path';
import { pipeline } from 'node:stream/promises';

import picomatch from 'picomatch';
import * as yauzl from 'yauzl';

import type { Readable } from 'node:stream';

const ZIP_PATH = 'assets/Assets.zip';
const OUTPUT_DIR = 'assets/hytale';
// Bundled with the worker (tracked in git), unlike Common/ which goes to R2.
const PARTICLE_MANIFEST_PATH = path.join(OUTPUT_DIR, 'Particles', 'CosmeticParticles.json');
const PARTICLE_DEFINITION_PATTERN = 'Server/Particles/**/*.{particlesystem,particlespawner}';

interface ParticleSystemJson {
	Spawners?: { SpawnerId?: string; }[];
}

interface ParticleSpawnerJson {
	Particle?: { Texture?: string; };
}

interface ParticleManifest {
	Systems: Record<string, ParticleSystemJson>;
	Spawners: Record<string, ParticleSpawnerJson>;
}

type Patchline = 'release' | 'pre-release';

function getDefaultInstallPath(patchline: Patchline): string {
	if (process.platform === 'win32') {
		return path.join('Hytale', 'install', patchline, 'package', 'game', 'latest', 'Assets.zip');
	}
	if (process.platform === 'linux') {
		return path.join('.local', 'share', 'Hytale', 'install', patchline, 'package', 'game', 'latest', 'Assets.zip');
	}
	throw new Error(`Unsupported platform: ${process.platform}`);
}

function parsePatchline(): Patchline {
	const arg = process.argv.find(entry => entry.startsWith('--patchline='));
	if (arg) {
		const value = arg.slice('--patchline='.length).toLowerCase();
		if (value === 'release' || value === 'pre-release') {
			return value;
		}
		throw new Error(`Invalid patchline "${value}". Use --patchline=release or --patchline=pre-release`);
	}
	// Check env as fallback
	const env = process.env.HYTALE_PATCHLINE?.toLowerCase();
	if (env === 'release' || env === 'pre-release') {
		return env;
	}
	return 'pre-release';
}

// Glob patterns for files to extract
// Add patterns here as needed
const GLOB_PATTERNS: string[] = [
	'Cosmetics/CharacterCreator/**.json',
	// Get all blocky models
	'Common/Characters/*.blockymodel',
	'Common/Characters/**/*.blockymodel',
	'Common/Characters/Animations/Default/Idle.blockyanim',
	// Get all character pngs
	'Common/Characters/*.png',
	'Common/Characters/**/*.png',
	// Same for Cosmetics folder
	'Common/Cosmetics/*.png',
	'Common/Cosmetics/**/*.png',
	'Common/Cosmetics/**/*.blockymodel',
	// All tint gradients
	'Common/TintGradients/*.png',
	'Common/TintGradients/**/*.png',
];

/**
 * Promisified version of yauzl.open
 */
function openZip(zipPath: string, options: yauzl.Options): Promise<yauzl.ZipFile> {
	return new Promise((resolve, reject) => {
		yauzl.open(zipPath, options, (err, zipfile) => {
			if (err) {
				reject(err);
			} else {
				resolve(zipfile as yauzl.ZipFile);
			}
		});
	});
}

/**
 * Promisified version of zipfile.openReadStream
 */
function openReadStream(zipfile: yauzl.ZipFile, entry: yauzl.Entry): Promise<Readable> {
	return new Promise((resolve, reject) => {
		zipfile.openReadStream(entry, (err, stream) => {
			if (err) {
				reject(err);
			} else {
				resolve(stream as Readable);
			}
		});
	});
}

/**
 * Normalize path separators to forward slashes for glob matching
 */
function normalizePath(filePath: string): string {
	return filePath.replaceAll('\\', '/');
}

/**
 * Check if a file path matches any of the glob patterns
 */
function matchesPattern(filePath: string, patterns: string[]): boolean {
	const normalizedPath = normalizePath(filePath);
	return picomatch.isMatch(normalizedPath, patterns, { windows: false });
}

/**
 * Visit every file entry in the zip, one at a time, so entries can be streamed without buffering the archive
 */
async function forEachZipEntry(
	zipPath: string,
	handleEntry: (zipfile: yauzl.ZipFile, entry: yauzl.Entry) => Promise<void>,
): Promise<void> {
	const zipfile = await openZip(zipPath, { lazyEntries: true });

	zipfile.readEntry();
	zipfile.on('entry', async (entry: yauzl.Entry) => {
		if (!/\/$/.test(entry.fileName)) {
			try {
				await handleEntry(zipfile, entry);
			} catch (error) {
				console.error(`✗ Error processing ${entry.fileName}:`, error);
			}
		}
		zipfile.readEntry();
	});

	await new Promise<void>((resolve, reject) => {
		zipfile.on('end', () => {
			resolve();
		});
		zipfile.on('error', (err) => {
			reject(err);
		});
	});
}

async function extractEntry(zipfile: yauzl.ZipFile, entry: yauzl.Entry): Promise<void> {
	const outputPath = path.join(OUTPUT_DIR, entry.fileName);
	await fs.mkdir(path.dirname(outputPath), { recursive: true });
	const readStream = await openReadStream(zipfile, entry);
	await pipeline(readStream, createWriteStream(outputPath));
	console.log(`✓ Extracted: ${entry.fileName}`);
}

async function readEntryText(zipfile: yauzl.ZipFile, entry: yauzl.Entry): Promise<string> {
	const readStream = await openReadStream(zipfile, entry);
	const chunks: Buffer[] = [];
	// yauzl streams never settle under async iteration or stream/consumers, so collect via events.
	await new Promise<void>((resolve, reject) => {
		readStream.on('data', (chunk: Buffer) => chunks.push(chunk));
		readStream.on('end', resolve);
		readStream.on('error', reject);
	});
	return Buffer.concat(chunks).toString('utf8');
}

/**
 * Build a manifest of the particle systems and spawners referenced by CharacterCreator cosmetics
 */
async function buildParticleManifest(
	systems: Map<string, ParticleSystemJson>,
	spawners: Map<string, ParticleSpawnerJson>,
): Promise<ParticleManifest> {
	const characterCreatorDir = path.join(OUTPUT_DIR, 'Cosmetics', 'CharacterCreator');
	const manifest: ParticleManifest = { Systems: {}, Spawners: {} };

	for (const fileName of await fs.readdir(characterCreatorDir)) {
		if (!fileName.endsWith('.json')) {
			continue;
		}
		const buffer = await fs.readFile(path.join(characterCreatorDir, fileName));
		const content: unknown = JSON.parse(buffer.toString('utf8'));
		if (!Array.isArray(content)) {
			continue;
		}
		for (const definition of content as { Particles?: { SystemId?: string; }[]; }[]) {
			for (const particle of definition.Particles ?? []) {
				const systemId = particle.SystemId;
				if (!systemId || manifest.Systems[systemId]) {
					continue;
				}
				const system = systems.get(systemId);
				if (!system) {
					console.warn(`Particle system not found: ${systemId}`);
					continue;
				}
				manifest.Systems[systemId] = system;
				for (const { SpawnerId: spawnerId } of system.Spawners ?? []) {
					const spawner = spawnerId ? spawners.get(spawnerId) : undefined;
					if (spawnerId && spawner) {
						manifest.Spawners[spawnerId] = spawner;
					} else {
						console.warn(`Particle spawner not found: ${spawnerId} (system ${systemId})`);
					}
				}
			}
		}
	}

	return manifest;
}

/**
 * Extract files from zip matching glob patterns
 */
async function extractAssets(): Promise<void> {
	const patchline = parsePatchline();
	console.log(`Using patchline: ${patchline}`);

	// Check if zip file exists
	let zipPathToUse = ZIP_PATH;
	try {
		await fs.access(ZIP_PATH);
	} catch {
		// Try seeing if the game is installed and use the default path for this patchline
		if (process.platform === 'win32') {
			try {
				const defaultPath = process.env.APPDATA;
				if (defaultPath) {
					const zipPath = path.join(defaultPath, getDefaultInstallPath(patchline));
					await fs.access(zipPath);
					console.log(`Using Windows zip file at: ${zipPath}`);
					zipPathToUse = zipPath;
				}
			} catch {
				throw new Error(`Zip file not found: ${zipPathToUse}. Please place Hytale Assets.zip in the assets/ folder or use --patchline=release or --patchline=pre-release.`);
			}
		} else if (process.platform === 'linux') {
			try {
				const zipPath = path.join(process.env.HOME || '~/', getDefaultInstallPath(patchline));
				await fs.access(zipPath);
				console.log(`Using Linux zip file at: ${zipPath}`);
				zipPathToUse = zipPath;
			} catch {
				throw new Error(`Zip file not found: ${zipPathToUse}. Please place Hytale Assets.zip in the assets/ folder or use --patchline=release or --patchline=pre-release.`);
			}
		} else {
			throw new Error(`Zip file not found: ${zipPathToUse}. Please place Hytale Assets.zip in the assets/ folder.`);
		}
	}

	// Ensure output directory exists
	await fs.mkdir(OUTPUT_DIR, { recursive: true });

	console.log(`Opening zip file: ${zipPathToUse}`);
	console.log(`Extracting files matching patterns: ${GLOB_PATTERNS.join(', ')}`);
	console.log(`Output directory: ${OUTPUT_DIR}\n`);

	let extractedCount = 0;
	const particleSystems = new Map<string, ParticleSystemJson>();
	const particleSpawners = new Map<string, ParticleSpawnerJson>();

	await forEachZipEntry(zipPathToUse, async (zipfile, entry) => {
		if (matchesPattern(entry.fileName, GLOB_PATTERNS)) {
			await extractEntry(zipfile, entry);
			extractedCount++;
		} else if (matchesPattern(entry.fileName, [PARTICLE_DEFINITION_PATTERN])) {
			const json: unknown = JSON.parse(await readEntryText(zipfile, entry));
			// Particle asset IDs are file names, so definitions are keyed by base name.
			const [id, extension] = path.posix.basename(entry.fileName).split('.');
			if (extension === 'particlesystem') {
				particleSystems.set(id, json as ParticleSystemJson);
			} else {
				particleSpawners.set(id, json as ParticleSpawnerJson);
			}
		}
	});

	const manifest = await buildParticleManifest(particleSystems, particleSpawners);
	await fs.mkdir(path.dirname(PARTICLE_MANIFEST_PATH), { recursive: true });
	await fs.writeFile(PARTICLE_MANIFEST_PATH, `${JSON.stringify(manifest, null, '\t')}\n`);
	console.log(`✓ Wrote ${PARTICLE_MANIFEST_PATH} (${Object.keys(manifest.Systems).length} systems)`);

	// Particle textures are only known once the manifest is resolved, so they need a second pass.
	const particleTextures = new Set<string>();
	for (const spawner of Object.values(manifest.Spawners)) {
		if (spawner.Particle?.Texture) {
			particleTextures.add(`Common/${spawner.Particle.Texture}`);
		}
	}
	if (particleTextures.size > 0) {
		await forEachZipEntry(zipPathToUse, async (zipfile, entry) => {
			if (particleTextures.has(normalizePath(entry.fileName))) {
				await extractEntry(zipfile, entry);
				extractedCount++;
			}
		});
	}

	console.log('\nExtraction complete!');
	console.log(`  Extracted: ${extractedCount} files`);
}

// Run the extraction
extractAssets().catch((error) => {
	console.error('Fatal error:', error);
	throw error;
});
