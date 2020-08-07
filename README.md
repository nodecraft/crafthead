# Crafthead

Crafthead is a highly scalable, serverless Minecraft avatar rendering
solution built on [Cloudflare Workers](https://workers.cloudflare.com).
It runs on any of Cloudflare's 200+ datacenters worldwide with minimal
latency.

**Note**: if you're a causal browser and wound up here, you probably want
to go to [Crafthead.net](https://crafthead.net), the public instance
of Crafthead I run.

## Features

* **Extremely fast**
* Supports UUID fetching (dashed or not dashed) and username lookups
* Renders avatars from 8px to 300px

## Rolling your own

### Step 1: Reconsider

Use the public instance I've made available. You don't have to pay a single
dime for it. I pick up all the costs. Plus, if you use it, you get speedy
performance all the time since it is more likely my version will be available
to run immediately compared to yours.

If, on the other hand, you're looking to _hack_ on Crafthead, then keep reading.

### Step 2: You want to do it?

This project uses `wrangler`, so make sure you have that installed. It is also
_strongly_ recommnded you purchase a Cloudflare Workers Unlimited plan, otherwise
it is possible you could run into CPU limit problems at larger resolutions.

You'll also want [Node.js](https://nodejs.org), the [Rust toolchain](https://www.rust-lang.org/learn/get-started)
and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) too.

Then:
* Copy `wrangler.toml.dev` to `wrangler.toml` and fill in your own `account_id`
* Use `yarn install` to install all the development dependencies
* Use `wrangler publish`. You're done!

### Notes on `wrangler preview`

If you're looking to test Crafthead using `wrangler preview` or `wrangler dev`,
you should use the special-cased usernames `MHF_Steve` and `char`, which return
the default "Steve" skin. If your code affects username lookups, however, you
should consider deploying to `workers.dev` instead.