/// <reference path="./mojang.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import PromiseGatherer from './promise_gather';
const STEVE_SKIN: ArrayBuffer = require('arraybuffer-loader!./steve.png');
declare var self: CloudflareWorkerGlobalScope;

interface MojangProfile {
    id: string;
    name: string;
    properties: MojangProfileProperty[];
}

interface MojangUsernameLookupResult {
    id: string;
    name: string;
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
    promiseGatherer: PromiseGatherer;

    constructor(promiseGatherer: PromiseGatherer) {
        this.promiseGatherer = promiseGatherer;
    }

    async retrieveSkin(uuid: string): Promise<Response> {
        // See if we already have the skin cached already.
        const cacheUrl = `https://mcavatar.steinborn.workers.dev/skin/${uuid}`
        const cachedSkin = await self.caches.default.match(new Request(cacheUrl))
        if (cachedSkin) {
            return cachedSkin;
        }

        // Otherwise we'll need to make 2 requests to get it. *sigh*
        const retrieved = await this.retrieveSkinFromMojang(uuid)
        this.promiseGatherer.push(caches.default.put(new Request(cacheUrl), retrieved.clone()))
        return retrieved
    }

    private async retrieveSkinFromMojang(uuid: string): Promise<Response> {
        const profile = await this.fetchMojangProfile(uuid);
        if (profile && profile.properties) {
            const skinTextureUrl = profile.properties
                .filter(property => property.name === 'textures')
                .map(this.extractUrlFromTexturesProperty)
                .pop()
            if (skinTextureUrl) {
                const textureResponse = await fetch(skinTextureUrl)
                if (!textureResponse.ok) {
                    throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`)
                }

                console.log("Successfully retrieved skin texture")
                return textureResponse
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return this.getSteveSkin()
    }

    getSteveSkin(): Response {
        return new Response(STEVE_SKIN)
    }

    async fetchMojangProfile(uuid: string): Promise<MojangProfile | null> {
        const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${uuid}`, {
            cf: {
                cacheTtl: 300
            }
        })

        if (profileResponse.status === 200) {
            const profile: MojangProfile = await profileResponse.json();
            this.promiseGatherer.push(this.saveUsernameCache(profile));
            return profile;
        } else if (profileResponse.status === 206) {
            return null;
        } else {
            throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`);
        }
    }

    private async lookupUsernameFromMojang(username: string): Promise<MojangUsernameLookupResult | undefined> {
        const body = JSON.stringify([username])
        const lookupResponse = await fetch('https://api.mojang.com/profiles/minecraft', {
            method: 'POST',
            body: body,
            headers: {
                'Content-Type': 'application/json'
            },
            cf: {
                cacheTtl: 300
            }
        })

        if (!lookupResponse.ok) {
            throw new Error(`Unable to retrieve profile from Mojang, http status ${lookupResponse.status}`);
        }

        const contents: MojangUsernameLookupResult[] | undefined = await lookupResponse.json();
        if (typeof contents === 'undefined' || contents.length === 0) {
            return undefined;
        }
        return contents[0];
    }

    async mapNameToUuid(name: string): Promise<string | undefined> {
        const url = this.getNameCacheUrl(name.toLocaleLowerCase('en-US'));
        const cachedName = await caches.default.match(url);
        if (cachedName) {
            return cachedName.text();
        }

        const mojangResponse = await this.lookupUsernameFromMojang(name);
        if (mojangResponse) {
            this.promiseGatherer.push(this.saveUsernameCache(mojangResponse));
            return mojangResponse.id;
        } else {
            return undefined;
        }
    }

    private async saveUsernameCache(profile: MojangUsernameLookupResult | MojangProfile): Promise<undefined> {
        const url = this.getNameCacheUrl(profile.name.toLocaleLowerCase('en-US'));
        return caches.default.put(url, new Response(profile.id, {
            headers: {
                'Cache-Control': 'public, max-age=3600'
            }
        }))
    }

    private getNameCacheUrl(name: string): string {
        return `https://api.mojang.com/profiles/minecraft/lookup-name/${name}`
    }

    private extractUrlFromTexturesProperty(property: MojangProfileProperty): string | undefined {
        const rawJson = atob(property.value);
        const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);
        console.log("Raw textures property: ", property);

        const textures = decoded.textures;
        return textures.SKIN && textures.SKIN.url;
    }

}