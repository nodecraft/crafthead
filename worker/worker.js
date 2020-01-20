addEventListener('fetch', event => {
  event.respondWith(handleRequest(event))
})

const STEVE_SKIN = "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"

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(event) {
  const request = event.request
  const interpreted = interpretRequest(request)
  if (!interpreted) {
    // We don't understand this request. Pass it straight to the origin (Amazon S3).
    return await fetch(request)
  }

  console.log("Request interpreted as ", interpreted)

  if (interpreted.identityType === 'username') {
    // Usernames are not yet supported.
    return new Response('username fetching not yet supported', { status: 400 })
  }

  // Catch top-level requests 
  const cacheUrl = getCacheUrl(request, interpreted)
  const cacheRequest = new Request(cacheUrl)
  const cachedResponse = await caches.default.match(cacheRequest)
  if (cachedResponse) {
    console.log("Request satisified from cache")
    return cachedResponse
  }

  // We failed to be lazy, so we'll have to actually fetch the skin.
  console.log("Request not satisified from cache. Going to do the computationally expensive stuff.")
  const skinResponse = await processRequest(interpreted)
  if (skinResponse.ok) {
    event.waitUntil(caches.default.put(cacheRequest, skinResponse.clone()))
  }
  return skinResponse
}

async function processRequest(interpreted) {
  switch (interpreted.requested) {
    case "avatar":
      return generateHead(interpreted.identity, interpreted.size)
    case "skin":
      return retrieveSkinAsResponse(interpreted.identity)
    default:
      return new Response('must request an avatar or a skin', { status: 400 })
  }
}

function getCacheUrl(request, interpreted) {
  const urlJs = new URL(request.url)
  return new URL(`${urlJs.protocol}//${urlJs.host}/${interpreted.requested}/${interpreted.identity}/${interpreted.size}`)
}

function interpretRequest(request) {
  const url = new URL(request.url)
  if (url.href.endsWith(".png")) {
    url.href = url.href.substring(0, url.href.length - 4)
  }

  let [requested, identity, size] = url.pathname.split('/').slice(1)
  size = parseInt(size, 10);
  if (!size) {
    size = 32 // default
  } else if (size < 8) {
    size = 8 // minimum size
  } else if (size > 300) {
    // In order to limit abuse, don't scale above 300px.
    size = 300
  }

  if (requested !== 'avatar' && requested !== 'skin') {
    return null
  }

  let identityType
  if (identity.length <= 16) {
    identityType = 'username'
  } else if (identity.length === 32) {
    identityType = 'uuid'
  } else if (identity.length === 36) {
    identity = identity.replace(/-/g, '')
    identityType = 'uuid'
  } else {
    return null
  }

  return { requested, identityType, identity, size }
}

async function generateHead(uuid, size) {
  const skin = await retrieveSkin(uuid)

  console.log("Generating head...")
  const { get_minecraft_head } = wasm_bindgen;
  await wasm_bindgen(wasm)
  const image = get_minecraft_head(skin, size)
  console.log("Head successfully generated.")

  return new Response(image, {
    status: 200,
    headers: {
      "Content-Type": "image/png",
      "Cache-Control": "max-age=86400"
    }
  })
}

// Gets a skin from the Mojang API for a specific UUID. Returns an Uint8Array. The specified UUID has to be
// in Mojang UUID format.
async function retrieveSkin(uuid) {
  const response = await retrieveSkinAsResponse(uuid)
  return new Uint8Array(await response.arrayBuffer())
}

// Gets a skin from the Mojang API for a specific UUID. Returns an Uint8Array. The response
// is a regular Response.
async function retrieveSkinAsResponse(uuid) {
  // See if we already have the skin cached already.
  const cacheUrl = new URL(`https://mcavatar.steinborn.workers.dev/skin/${uuid}`)
  const cachedSkin = await caches.default.match(new Request(cacheUrl))
  if (cachedSkin) {
    return cachedSkin;
  }

  const retrieved = await backendRetrieveSkin(uuid)
  await caches.default.put(new Request(cacheUrl), retrieved.clone())
  return retrieved
}

async function backendRetrieveSkin(uuid) {
  const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${uuid}`)
  if (!profileResponse.ok) {
    throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`)
  }

  if (profileResponse.status === 200) {
    const profile = await profileResponse.json()
    if (profile.properties) {
      const skinTextureUrl = profile.properties
        .filter(property => property.name === 'textures')
        .map(property => readTexturesProperty(property.value))
        .pop()
      if (skinTextureUrl) {
        const textureResponse = await fetch(skinTextureUrl)
        if (!textureResponse.ok) {
          throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`)
        }
        console.log("Successfully retrieved skin texture.")
        return new Response(await textureResponse.arrayBuffer())
      }
    }
  }

  console.log("Invalid properties found! Falling back to Steve skin.")
  return new Response(base64ToUint8(STEVE_SKIN))
}

function readTexturesProperty(property) {
  const rawJson = atob(property)
  const decoded = JSON.parse(rawJson)
  console.log("Raw textures property: " + property)

  const textures = decoded.textures
  return textures.SKIN && textures.SKIN.url
}

function base64ToUint8(base64) {
  var raw = atob(base64);
  var rawLength = raw.length;
  var array = new Uint8Array(new ArrayBuffer(rawLength));

  for (i = 0; i < rawLength; i++) {
    array[i] = raw.charCodeAt(i);
  }
  return array
}