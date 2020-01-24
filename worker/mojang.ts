/// <reference path="./mojang.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import PromiseGatherer from './promise_gather';
import { IdentityKind } from './request';
const STEVE_SKIN: ArrayBuffer = require('base64-js').toByteArray("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")
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

interface SkinResponse {
    response: Response;
    profile: MojangProfile | null;
}

const SKIN_CACHE_BASE_URL = 'https://mcavatar.steinborn.workers.dev/skin'

export default class MojangRequestService {
    promiseGatherer: PromiseGatherer;

    constructor(promiseGatherer: PromiseGatherer) {
        this.promiseGatherer = promiseGatherer;
    }

    async retrieveSkin(identity: string, identityType: IdentityKind): Promise<Response> {
        // Minotar special-cases these
        if (identityType === IdentityKind.Username && (identity === 'char' || identity === 'MHF_Steve')) {
            return this.getSteveSkin();
        }

        // See if we already have the skin cached already.
        const cacheUrl = `${SKIN_CACHE_BASE_URL}/${identity}`
        const cachedSkin = await self.caches.default.match(new Request(cacheUrl))
        if (cachedSkin) {
            return cachedSkin;
        }

        // Otherwise we'll need to make 2 requests to get it. *sigh*
        const retrieved = await this.retrieveSkinFromMojang(identity, identityType)

        // Cache it too
        const storedRequest = new Response(retrieved.response.body, {
            headers: {
                'Cache-Control': 'public, max-age=172800'
            }
        })

        const cacheUrls: Set<string> = new Set()
        cacheUrls.add(cacheUrl);
        if (retrieved.profile) {
            cacheUrls.add(`${SKIN_CACHE_BASE_URL}/${retrieved.profile.id}`)
            cacheUrls.add(`${SKIN_CACHE_BASE_URL}/${retrieved.profile.name.toLocaleLowerCase('en-US')}`)
        }

        const cachePromises = []
        for (const putCacheUrl of cacheUrls) {
            cachePromises.push(caches.default.put(putCacheUrl, storedRequest.clone()))
        }
        await Promise.all(cachePromises)
        return storedRequest
    }

    private async retrieveSkinFromMojang(identity: string, identityType: IdentityKind): Promise<SkinResponse> {
        const profile = await this.fetchMojangProfile(identity, identityType);
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
                return {
                    response: textureResponse,
                    profile
                }
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return {
            response: this.getSteveSkin(),
            profile: null
        }
    }

    getSteveSkin(): Response {
        return new Response(STEVE_SKIN)
    }

    async fetchMojangProfile(identity: string, identityType: IdentityKind): Promise<MojangProfile | null> {
        const doLookup = (id: string): Promise<Response> => {
            return fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${id}`, {
                cf: {
                    cacheEverything: true,
                    cacheTtl: 3600
                }
            })
        }

        let profilePromise: Promise<Response>
        switch (identityType) {
            case IdentityKind.Uuid:
                profilePromise = doLookup(identity)
                break
            case IdentityKind.Username:
                profilePromise = this.lookupUsernameFromMojang(identity)
                    .then((result) => {
                        if (!result) {
                            return new Response('', { status: 206 })
                        }
                        return doLookup(result.id);
                    });
                break;
        }

        const profileResponse = await profilePromise;
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
                cacheEverything: true,
                cacheTtl: 3600
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
                'Cache-Control': 'public, max-age=86400'
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