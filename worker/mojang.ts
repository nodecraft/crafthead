/// <reference path="./mojang.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import PromiseGatherer from './promise_gather';
import { IdentityKind } from './request';
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