/**
 * Upload all Hytale assets from assets/hytale/ to R2 bucket
 *
 * This script recursively reads all files from assets/hytale/ and uploads them
 * to the Cloudflare R2 bucket via the /upload-asset HTTP endpoint.
 * Files are uploaded in parallel for better performance.
 */

import { readFile, readdir, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ASSETS_DIR = path.resolve(__dirname, '../assets/hytale');

// Default to localhost:8787 (wrangler dev default), but allow override via env
const UPLOAD_ENDPOINT = process.env.UPLOAD_ENDPOINT || 'http://localhost:8787/upload-asset';

/**
 * Recursively get all files from a directory
 */
async function getAllFiles(dirPath: string): Promise<string[]> {
	const files: string[] = [];
	const entries = await readdir(dirPath, { withFileTypes: true });

	for (const entry of entries) {
		const fullPath = path.join(dirPath, entry.name);
		if (entry.isDirectory()) {
			const subFiles = await getAllFiles(fullPath);
			files.push(...subFiles);
		} else {
			files.push(fullPath);
		}
	}

	return files;
}

/**
 * Get content type based on file extension
 */
function getContentType(filePath: string): string {
	const ext = path.extname(filePath).toLowerCase();
	switch (ext) {
		case '.png': {
			return 'image/png';
		}
		case '.json': {
			return 'application/json';
		}
		// Technically these are just json files
		case '.blockymodel':
		case '.blockyanim': {
			return 'application/json';
		}
		default: {
			return 'application/octet-stream';
		}
	}
}

/**
 * Upload a single file to R2 via HTTP endpoint
 */
async function uploadFile(absolutePath: string, relativePath: string): Promise<{ success: boolean; error?: string; }> {
	try {
		// Read file content
		const fileContent = await readFile(absolutePath);
		const contentType = getContentType(relativePath);

		// Create form data
		const formData = new FormData();
		formData.append('filePath', relativePath);
		formData.append('file', new Blob([fileContent], { type: contentType }));
		formData.append('contentType', contentType);

		const response = await fetch(UPLOAD_ENDPOINT, {
			method: 'POST',
			body: formData,
		});

		if (!response.ok) {
			const errorText = await response.text();
			return { success: false, error: `HTTP ${response.status}: ${errorText}` };
		}

		const result = await response.json() as { success: boolean; error?: string; };
		return result;
	} catch (error) {
		return { success: false, error: (error as Error).message };
	}
}

/**
 * Main upload function with parallel uploads
 */
async function uploadAssets(): Promise<void> {
	console.log(`Scanning assets in ${ASSETS_DIR}...`);
	console.log(`Using upload endpoint: ${UPLOAD_ENDPOINT}\n`);

	// Check if directory exists
	try {
		const dirStat = await stat(ASSETS_DIR);
		if (!dirStat.isDirectory()) {
			throw new Error(`Path ${ASSETS_DIR} is not a directory`);
		}
	} catch (err) {
		throw new Error(`Assets directory not found: ${ASSETS_DIR}. Error: ${(err as Error).message}`);
	}

	// Get all files recursively
	const allFiles = await getAllFiles(ASSETS_DIR);
	console.log(`Found ${allFiles.length} files to upload.\n`);

	// Convert to relative paths
	const filesToUpload = allFiles.map((filePath) => {
		return {
			absolutePath: filePath,
			relativePath: path.relative(ASSETS_DIR, filePath).replaceAll('\\', '/'),
		};
	});

	const stats = {
		total: filesToUpload.length,
		successful: 0,
		failed: 0,
		errors: [] as Array<{ file: string; error: string; }>,
	};

	// Upload files in parallel batches (throttled to avoid overwhelming wrangler)
	const CONCURRENT_UPLOADS = 20; // Reduced from 20 to avoid overwhelming wrangler
	let completed = 0;

	console.log(`Uploading ${stats.total} files with up to ${CONCURRENT_UPLOADS} concurrent uploads...\n`);

	for (let i = 0; i < filesToUpload.length; i += CONCURRENT_UPLOADS) {
		const batch = filesToUpload.slice(i, i + CONCURRENT_UPLOADS);

		const results = await Promise.allSettled(
			batch.map(async ({ absolutePath, relativePath }) => {
				const result = await uploadFile(absolutePath, relativePath);
				completed++;

				// Update progress
				process.stdout.write(`\r[${completed}/${stats.total}] Uploading... ${relativePath.slice(0, 60)}${relativePath.length > 60 ? '...' : ''}`);

				return { relativePath, result };
			}),
		);

		// Process results
		for (const settled of results) {
			if (settled.status === 'fulfilled') {
				const { relativePath, result } = settled.value;
				if (result.success) {
					stats.successful++;
				} else {
					stats.failed++;
					stats.errors.push({
						file: relativePath,
						error: result.error || 'Unknown error',
					});
				}
			} else {
				stats.failed++;
				const relativePath = settled.reason?.relativePath || 'unknown';
				stats.errors.push({
					file: relativePath,
					error: settled.reason?.message || 'Promise rejected',
				});
			}
		}

		// Small delay between batches to avoid overwhelming wrangler
		if (i + CONCURRENT_UPLOADS < filesToUpload.length) {
			await new Promise((resolve) => {
				setTimeout(resolve, 100); // 100ms delay between batches
			});
		}
	}

	// Clear progress line
	process.stdout.write('\r' + ' '.repeat(80) + '\r');

	// Print summary
	console.log('\n' + '='.repeat(50));
	console.log('Upload Summary:');
	console.log(`  Total files: ${stats.total}`);
	console.log(`  Successful: ${stats.successful}`);
	console.log(`  Failed: ${stats.failed}`);

	if (stats.errors.length > 0) {
		console.log('\nErrors:');
		for (const error of stats.errors.slice(0, 20)) {
			console.log(`  - ${error.file}: ${error.error}`);
		}
		if (stats.errors.length > 20) {
			console.log(`  ... and ${stats.errors.length - 20} more errors`);
		}
	}

	if (stats.failed > 0) {
		throw new Error(`Upload failed for ${stats.failed} file(s)`);
	}
}

// Run the upload
uploadAssets().catch((error) => {
	console.error('Fatal error:', error);
	throw error;
});
