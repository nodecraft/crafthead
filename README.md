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

## Overall breakdown

Crafthead is split into two primary components: the request handler frontend,
written in TypeScript, and the image generation logic, written in Rust and
invoked by TypeScript by way of WebAssembly. This means it is possible to use
Crafthead's image generation from within the browser, provided you had another
method of loading the skins from the browser. However, the most convenient way
to use Crafthead is to use the publicly-hosted instance that does the rendering
on Cloudflare's extensive edge network.

The request handler (located under the `worker` directory) receives every request
to Crafthead. It first parses each request to figure out what it is asking for. If
Crafthead can't determine what the request is, it will use Cloudflare Workers Sites
to serve the index page or a 404 page. Once it has determined what is to be retrieved
(and misses the top-level cache), each request is "normalized" (essentially, this
entails changing all username requests to use UUIDs instead) and the skin is looked
up (all stages of this are cached in Workers KV and in the local cache for speed).

Skin and profile requests are fully serviced by the TypeScript request handler, however if
an avatar, helm, or cube render (with others TBD) are requested, the skin, size, and render
request are passed to the image generation logic written in Rust (located in `src`). The Rust
component determines what is to be rendered, loads the skin's PNG using the `image` crate,
renders the desired image (using the primitives of the `image` crate and the `1mageproc` crate),
saves the PNG into memory, and returns it to the TypeScript request handler to be sent to the client.
This is why Crafthead requires a paid Cloudflare Workers plan: image handling is computationally
expensive, and the 10ms CPU limit is insufficient for all but the smallest requests.

## Rolling your own

### Step 1: Reconsider

Use the public instance I've made available. You don't have to pay a single
dime for it. I pick up all the costs. Plus, if you use it, you get speedy
performance all the time since it is more likely my instance will be available
to run immediately compared to yours.

If, on the other hand, you're looking to _hack_ on Crafthead, then keep reading.

### Step 2: You want to do it?

You will need to have the following:

* A **paid** Cloudflare Workers plan. The Worker can be Bundled or Unbound,
  though the production setup is anticipated to be on the Unbound plan in the future.
  This is because Crafthead uses Workers KV and can easily exceed the CPU limits on
  the free plan.
* The `wrangler` CLI.
* [Node.js](https://nodejs.org).
* [Rust](https://www.rust-lang.org/learn/get-started).
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

Then:

* Copy `wrangler.toml.dev` to `wrangler.toml` and fill in your own `account_id`.
  in your configuration with the output from the command.
* Use `npm ci` to install all the development dependencies
* You may need to remove the `services` section from `wrangler.toml`
* Use `wrangler publish`. You're done!

### Notes on `wrangler preview`

If you're looking to test Crafthead using `wrangler preview` or `wrangler dev`,
you should use the special-cased usernames `MHF_Steve` and `char`, which return
the default "Steve" skin. If your code affects username lookups, however, you
should consider deploying to `workers.dev` instead.
