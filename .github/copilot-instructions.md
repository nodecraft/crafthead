# Crafthead - Minecraft Avatar Rendering Service

Crafthead is a high-performance, serverless Minecraft avatar rendering service built on Cloudflare Workers. It combines TypeScript worker code with Rust WebAssembly for image processing to deliver fast avatar rendering from any of Cloudflare's 200+ datacenters worldwide.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Environment Setup and Dependencies
- Install Node.js 22+ (required by package.json engines):
  - `curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs`
  - Or use nvm: `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash && nvm install 22 && nvm use 22`
- Ensure Rust toolchain is available: `rustc --version && cargo --version`
- Install wasm-pack: `cargo install wasm-pack`
- Verify wasm-pack installation: `wasm-pack --version`

### Bootstrap, Build, and Test
- `npm ci` -- installs dependencies. Takes ~20 seconds. NEVER CANCEL.
- `npm run build` -- compiles Rust to WebAssembly and fixes output. **Takes 2+ minutes. NEVER CANCEL. Set timeout to 180+ seconds.**
- `npm run test` -- runs Vitest and Cargo tests. Takes ~20 seconds total. NEVER CANCEL.
  - Note: Some tests may fail in network-restricted environments due to DNS lookups to minecraft.net and playerdb.co. This is expected.
- `cargo test` -- runs Rust tests only. Takes ~20 seconds. NEVER CANCEL.

### Development Server
- **ALWAYS run the build step first before starting the dev server**
- `npm run dev` -- starts Cloudflare Workers development server on http://localhost:8787
- The server rebuilds automatically when files change
- Network warnings about workers.cloudflare.com are normal and don't prevent functionality

### Linting and Code Quality
- `npm run lint` -- runs ESLint for TypeScript/JavaScript and Rust fmt/clippy. Takes ~20 seconds. NEVER CANCEL.
- `npm run lint:js:fix` -- auto-fixes JavaScript/TypeScript linting issues
- `npm run lint:rs:fix` -- auto-fixes Rust formatting and some clippy issues  
- `npm run check-types` -- TypeScript type checking
- Always run `npm run lint` before committing or the CI will fail

## Validation

### Manual Testing Scenarios
After making changes, **ALWAYS** manually test the application functionality:

**Basic Functionality:**
- Test home page: `curl -I http://localhost:8787/` (should return HTML)
- Test avatar rendering: `curl -I http://localhost:8787/avatar/MHF_Steve/64` (should return image/png)
- Test different sizes: `curl -I http://localhost:8787/avatar/MHF_Steve/128` (should return image/png)

**Special Test Accounts:**
- Use `MHF_Steve` and `char` for local testing - these return the default "Steve" skin
- Example: `http://localhost:8787/avatar/MHF_Steve/64`

**Avatar Types to Test:**
- Avatar: `/avatar/MHF_Steve/64`
- Bust: `/bust/MHF_Steve/64` 
- Body: `/body/MHF_Steve/64`
- Helm: `/helm/MHF_Steve/64`
- Cube: `/cube/MHF_Steve/64`
- Skin: `/skin/MHF_Steve`
- Profile: `/profile/MHF_Steve`

**Identity Types:**
- Username: `MHF_Steve` (up to 16 characters)
- UUID without dashes: `ef6134805b6244e4a4467fbe85d65513` (32 characters)
- UUID with dashes: `ef613480-5b62-44e4-a446-7fbe85d65513` (36 characters)
- Texture ID: 64-character hex string

### Size Constraints
- Minimum size: 8px (requests for smaller sizes are clamped to 8px)
- Maximum size: 300px (requests for larger sizes are clamped to 300px)
- Default size: 180px (when no size specified)

### Expected Response Formats
- Images: `Content-Type: image/png`
- Profiles: `Content-Type: application/json`
- CORS headers: `Access-Control-Allow-Origin: *`
- Cache headers: `Cache-Control: max-age=14400`

## Key Projects and Architecture

### Core Components
- **`src/worker/`** - TypeScript Cloudflare Worker handling HTTP requests, caching, and orchestration
- **`src/rust/`** - Rust image processing library compiled to WebAssembly for high-performance rendering
- **`src/website/`** - Static website files served by Cloudflare Workers Sites
- **`pkg/`** - Generated WebAssembly output (created by build, not committed to git)

### Important Files
- **`src/worker/index.ts`** - Main worker entry point and request handler
- **`src/worker/request.ts`** - Request parsing and validation logic
- **`src/worker/services/mojang/`** - Mojang API integration for profile/skin lookups
- **`src/rust/lib.rs`** - Rust WebAssembly entry point for image rendering
- **`wrangler.toml`** - Cloudflare Workers configuration
- **`Cargo.toml`** - Rust project configuration
- **`helper/fix-wasm-pack-output.ts`** - Post-build script to fix wasm-pack output for Workers

### Request Flow
1. HTTP request arrives at Worker
2. Request parsed to determine type (avatar, bust, profile, etc.)
3. If cached, return cached response
4. Otherwise, fetch profile/skin from Mojang API
5. For image requests, pass to Rust WASM for rendering
6. Cache and return response

### Testing Infrastructure
- **Vitest** with `@cloudflare/vitest-pool-workers` for integration testing
- **Rust unit tests** for image processing logic
- Tests validate various avatar types, sizes, and identity formats

## Common Tasks and Outputs

### Repository Root Structure
```
.
├── README.md                 # Project documentation
├── package.json             # Node.js dependencies and scripts
├── Cargo.toml              # Rust project configuration
├── wrangler.toml           # Cloudflare Workers config
├── src/                    # Source code
│   ├── worker/            # TypeScript worker code
│   ├── rust/              # Rust WebAssembly code
│   └── website/           # Static website files
├── test/                   # Test files
├── helper/                 # Build helper scripts
└── pkg/                    # Generated WASM (build output)
```

### Build Process Details
1. `wasm-pack build --release -t nodejs` compiles Rust to WebAssembly
2. `helper/fix-wasm-pack-output.ts` modifies the generated JS for Workers compatibility
3. Output goes to `pkg/` directory with `.wasm` and `.js` files

### Performance Notes
- **CRITICAL BUILD TIMING**: Build takes 2+ minutes due to Rust compilation. NEVER CANCEL builds.
- First build downloads and compiles many Rust dependencies
- Subsequent builds are faster (~30 seconds) if dependencies unchanged
- wasm-pack may install wasm-bindgen CLI on first run, adding time

### Deployment
- Production deploys to `crafthead.net` via Cloudflare Workers
- Development uses `wrangler dev` for local testing
- Configuration in `wrangler.toml` with environment-specific settings