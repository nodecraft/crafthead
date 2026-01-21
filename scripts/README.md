# Asset Upload Scripts

This directory contains scripts for managing Hytale assets.

## Scripts

### extract-hytale-assets.ts
Extracts Hytale assets from the official Assets.zip file.

**Usage:**
1. Place `Hytale Assets.zip` in the `assets/` directory at the project root
   - On Windows, the script can automatically find the zip at `%APPDATA%\Hytale\install\release\package\game\latest\Assets.zip`
2. Run: `npm run extract-hytale-assets`
3. Assets are extracted to `assets/hytale/`

### upload-assets-to-r2.ts
Uploads Hytale assets to Cloudflare R2.

**Usage:**

#### HTTP Method (Default)
Uses the HTTP endpoint, requiring a running Wrangler dev server:
```bash
npm run upload-assets-to-r2
```

This uploads to `http://localhost:8787/upload-asset` by default. You can override with:
```bash
UPLOAD_ENDPOINT=http://your-endpoint/upload-asset npm run upload-assets-to-r2
```

#### S3 Method
Uploads directly to R2 using the S3-compatible protocol:

1. Copy `.env.example` to `.env`
2. Configure your R2 credentials:
   ```env
   UPLOAD_METHOD=s3
   R2_ENDPOINT=https://<your-account-id>.r2.cloudflarestorage.com
   R2_ACCESS_KEY_ID=your-access-key-id
   R2_SECRET_ACCESS_KEY=your-secret-access-key
   R2_BUCKET=hytale-assets
   R2_KEY_PREFIX=
   ```
3. Run: `npm run upload-assets-to-r2`

**Note on Paths:**
- By default, files are uploaded to the bucket root with paths like `Common/Characters/skin.png`
- If files were previously uploaded with an incorrect prefix (e.g., `hytale-assets/`), set `R2_KEY_PREFIX=` to upload correctly
- To add a prefix, set `R2_KEY_PREFIX=assets/` (includes trailing slash) to get paths like `assets/Common/Characters/skin.png`

#### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `UPLOAD_METHOD` | Upload method: `http` or `s3` | `http` |
| `UPLOAD_ENDPOINT` | HTTP endpoint URL (HTTP method only) | `http://localhost:8787/upload-asset` |
| `R2_ENDPOINT` | R2 S3-compatible endpoint (S3 method only) | - |
| `R2_ACCESS_KEY_ID` | R2 access key ID (S3 method only) | - |
| `R2_SECRET_ACCESS_KEY` | R2 secret access key (S3 method only) | - |
| `R2_BUCKET` | R2 bucket name (S3 method only) | `hytale-assets` |
| `R2_KEY_PREFIX` | Optional prefix for S3 keys (S3 method only) | `` (empty) |

**Performance:**
- Uploads up to 20 files in parallel
- Batches uploads with 100ms delays to avoid overwhelming the server
- Shows real-time progress during upload
- Prints a summary with success/failure counts and any errors

**Getting R2 Credentials:**
1. Go to Cloudflare Dashboard > R2 > Your Account
2. Create an API Token with R2 permissions
3. Or create R2 API credentials via: R2 > Manage R2 API Tokens > Create API Token
