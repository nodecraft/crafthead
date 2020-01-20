/// <reference path="./mojang.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
const STEVE_SKIN: ArrayBuffer = require('arraybuffer-loader!./steve.png');
declare var self: CloudflareWorkerGlobalScope;

interface MojangProfile {
    id: string;
    name: string;
    properties: MojangProfileProperty[];
}

interface MojangProfileProperty {
    name: string;
    value: string;
}

interface MojangTexturePropertyValue {
    textures: MojangTextureUrls;
}

interface MojangTextureUrls {
    SKIN: { url: string } | undefined;
    CAPE: { url: string } | undefined;
}

export default class MojangRequestService {
    async retrieveSkin(uuid: string): Promise<Response> {
        // See if we already have the skin cached already.
        const cacheUrl = `https://mcavatar.steinborn.workers.dev/skin/${uuid}`
        const cachedSkin = await self.caches.default.match(new Request(cacheUrl))
        if (cachedSkin) {
            return cachedSkin;
        }

        const retrieved = await this.retrieveSkinFromMojang(uuid)
        await caches.default.put(new Request(cacheUrl), retrieved.clone())
        return retrieved
    }

    private async retrieveSkinFromMojang(uuid: string): Promise<Response> {
        const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${uuid}`)
        if (!profileResponse.ok) {
            throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`)
        }

        if (profileResponse.status === 200) {
            const profile: MojangProfile = await profileResponse.json()
            if (profile.properties) {
                const skinTextureUrl = profile.properties
                    .filter(property => property.name === 'textures')
                    .map(this.extractUrlFromTexturesProperty)
                    .pop()
                if (skinTextureUrl) {
                    const textureResponse = await fetch(skinTextureUrl)
                    if (!textureResponse.ok) {
                        throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`)
                    }

                    const buffer = await textureResponse.arrayBuffer()
                    console.log("Successfully retrieved skin texture of ", buffer.byteLength, " bytes.")
                    return new Response(buffer)
                }
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return new Response(STEVE_SKIN)
    }

    private extractUrlFromTexturesProperty(property: MojangProfileProperty): string | undefined {
        const rawJson = atob(property.value);
        const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);
        console.log("Raw textures property: ", property);

        const textures = decoded.textures;
        return textures.SKIN && textures.SKIN.url;
    }

}