# Crafthead

Crafthead is a highly scalable, serverless Minecraft avatar rendering
solution built on [Cloudflare Workers](https://workers.cloudflare.com).
It runs on any of Cloudflare's 200+ datacenters worldwide with minimal
latency.

**Note**: if you're a causal browser and wound up you, you probably want
to go to [Crafthead.net](https://crafthead.net), the public instance
of Crafthead I run.

## Features

* **Extremely fast**:
  * Avatar generation at 300px in ~50ms (compare to 68ms for Minotar and 65ms for Crafatar)
  * Avatar generation at 64px in ~23ms (compare to 54ms for Minotar and 57ms for Crafatar)
  * Most of the benefit can be given to the fact Craftheads runs near the user
  * Note: the skin the avatar was generated from was always in cache
* Supports UUID fetching (dashed or not dashed), username lookups coming soon
* Renders avatars from 8px to 300px

## Rolling your own

### Step 1: Don't.

Use the public instance I've made available. You don't have to pay a single
dime for it. I pick up all the costs.

If, on the other hand, you're looking to _hack_ on Crafthead, then keep reading.

### Step 2: You want to do it?

This project uses `wrangler`, so make sure you have that installed. It is also
_strongly_ recommnded you purchase a Cloudflare Workers Unlimited plan, otherwise
it is possible you could run into CPU limit problems at larger resolutions.

You'll also want the [Rust toolchain](https://www.rust-lang.org/learn/get-started)
and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) too.

All you need is to copy `wrangler.toml.dev` to `wrangler.toml`, fill in your own
`account_id`, and use `wrangler publish`. That's all there is to it!

### Notes on `wrangler preview`

Due to Mojang API rate limits, it is not possible to use `wrangler preview` to
test Crafthead. You are better off deploying to `workers.dev` instead. In the
future, a "fake" mode may be introduced where only one head can be scaled as
needed.