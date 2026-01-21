# Assets

This directory contains extracted game assets used by Crafthead.

## Hytale Assets

To extract Hytale assets, use the `extract-hytale-assets.ts` script.

### Prerequisites

- Node.js 24 or higher
- Hytale Assets.zip file

### Extraction Methods

#### Method 1: Place zip file in assets folder (Recommended)

1. Place `Hytale Assets.zip` in the `assets/` folder
2. Run the extraction script:
   ```bash
   npm run extract-hytale-assets
   ```

#### Method 2: Automatic Windows detection

On Windows, the script will automatically detect if Hytale is installed and use the Assets.zip from the default installation location:
- `%APPDATA%\Hytale\install\release\package\game\latest\Assets.zip`

Simply run:
```bash
npm run extract-hytale-assets
```

### Configuring Extraction Patterns

The script extracts files based on glob patterns defined in `scripts/extract-hytale-assets.ts`. To modify which files are extracted, edit the `GLOB_PATTERNS` array in that file.

Current patterns extract:
- Character Creator JSON files
- Blocky model files (`.blockymodel`)
- Character PNG textures
- Cosmetics PNG textures
- Tint gradient PNG files

### Output

Extracted files are placed in `assets/hytale/` preserving the original directory structure from the zip file.
