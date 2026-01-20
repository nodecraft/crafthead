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

const ZIP_PATH = 'assets/Hytale Assets.zipasd';
const WINDOWS_ZIP_PATH = '\\Hytale\\install\\release\\package\\game\\latest\\Assets.zip';
const OUTPUT_DIR = 'assets/hytale';

// Glob patterns for files to extract
// Add patterns here as needed
const GLOB_PATTERNS: string[] = [
	'Cosmetics/CharacterCreator/**.json',
	// Get all blocky models
	'Common/Characters/*.blockymodel',
	'Common/Characters/**/*.blockymodel',
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
				resolve(zipfile!);
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
				resolve(stream!);
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
 * Extract files from zip matching glob patterns
 */
async function extractAssets(): Promise<void> {
	// Check if zip file exists
	let zipPathToUse = ZIP_PATH;
	try {
		await fs.access(ZIP_PATH);
	} catch {
		// Try seeing if the game is installed on Windows and use the default path
		if (process.platform === 'win32') {
			try {
				const defaultPath = process.env.APPDATA;
				if (defaultPath) {
					const zipPath = path.join(defaultPath, WINDOWS_ZIP_PATH);
					await fs.access(zipPath);
					console.log(`Using Windows zip file at: ${zipPath}`);
					zipPathToUse = zipPath;
				}
			} catch {
				throw new Error(`Zip file not found: ${zipPathToUse}. Please place Hytale Assets.zip in the assets/ folder.`);
			}
		} else {
			throw new Error(`Zip file not found: ${zipPathToUse}. Please place Hytale Assets.zip in the assets/ folder.`);
		}
	}

	// Ensure output directory exists
	await fs.mkdir(OUTPUT_DIR, { recursive: true });

	// Validate glob patterns
	if (GLOB_PATTERNS.length === 0) {
		console.warn('Warning: No glob patterns defined. No files will be extracted.');
		console.warn('Please add patterns to GLOB_PATTERNS array in the script.');
		return;
	}

	console.log(`Opening zip file: ${zipPathToUse}`);
	console.log(`Extracting files matching patterns: ${GLOB_PATTERNS.join(', ')}`);
	console.log(`Output directory: ${OUTPUT_DIR}\n`);

	// Open zip file
	const zipfile = await openZip(zipPathToUse, { lazyEntries: true });

	let extractedCount = 0;
	let skippedCount = 0;

	// Process each entry in the zip
	zipfile.readEntry();
	zipfile.on('entry', async (entry: yauzl.Entry) => {
		// Skip directories
		if (/\/$/.test(entry.fileName)) {
			zipfile.readEntry();
			return;
		}

		// Check if entry matches any glob pattern
		if (!matchesPattern(entry.fileName, GLOB_PATTERNS)) {
			skippedCount++;
			zipfile.readEntry();
			return;
		}

		try {
			// Determine output path
			const outputPath = path.join(OUTPUT_DIR, entry.fileName);
			const outputDir = path.dirname(outputPath);

			// Create parent directories if needed
			await fs.mkdir(outputDir, { recursive: true });

			// Stream entry to output file
			const readStream = await openReadStream(zipfile, entry);
			const writeStream = createWriteStream(outputPath);

			await pipeline(readStream, writeStream);

			extractedCount++;
			console.log(`✓ Extracted: ${entry.fileName}`);

			// Read next entry
			zipfile.readEntry();
		} catch (error) {
			console.error(`✗ Error extracting ${entry.fileName}:`, error);
			zipfile.readEntry();
		}
	});

	// Wait for zip processing to complete
	await new Promise<void>((resolve, reject) => {
		zipfile.on('end', () => {
			resolve();
		});
		zipfile.on('error', (err) => {
			reject(err);
		});
	});

	console.log('\nExtraction complete!');
	console.log(`  Extracted: ${extractedCount} files`);
	console.log(`  Skipped: ${skippedCount} files`);
}

// Run the extraction
extractAssets().catch((error) => {
	console.error('Fatal error:', error);
	throw error;
});
