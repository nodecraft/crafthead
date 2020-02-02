/// <reference path="./mojang.d.ts">

import PromiseGatherer from '../promise_gather';
import { IdentityKind } from '../request';
import { STEVE_SKIN } from '../data';
import { CloudflareCacheService, ArrayBufferCloudflareResponseMapper } from './cache/cloudflare';
import MemoryCacheService from './cache/memory';
import ResponseCacheService from './cache/response_helper';

const MOJANG_API_TTL = 86400

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

export default class MojangRequestService {
    skinCache: CacheService<Response>;

    constructor() {
        this.skinCache = new ResponseCacheService(
            new MemoryCacheService(),
            new CloudflareCacheService('skin', new ArrayBufferCloudflareResponseMapper())
        );
    }

    async storeInCaches(identity: string, response: SkinResponse): Promise<any> {
        const cacheUrls: Set<string> = new Set()
        cacheUrls.add(identity);
        if (response.profile) {
            cacheUrls.add(`${response.profile.id}`)
            cacheUrls.add(`${response.profile.name.toLocaleLowerCase('en-US')}`)
        }

        const cachePromises = []
        for (const putCacheUrl of cacheUrls) {
            cachePromises.push(this.skinCache.put(putCacheUrl, response.response.clone()))
        }
        return Promise.all(cachePromises)
    }

    async retrieveSkin(identity: string, identityType: IdentityKind): Promise<Response> {
        // Minotar special-cases these
        if (identityType === IdentityKind.Username && (identity === 'char' || identity === 'MHF_Steve')) {
            return new Response(STEVE_SKIN);
        }

        // See if we already have the skin cached already.
        const cachedSkin = await this.skinCache.find(identity);
        if (cachedSkin) {
            return cachedSkin;
        }

        // Otherwise we'll need to make 2 requests to get it. *sigh*
        const promiseGatherer = new PromiseGatherer()
        const retrieved = await this.retrieveSkinFromMojang(identity, identityType, promiseGatherer)

        // Cache it too
        await Promise.all([this.storeInCaches(identity, retrieved), promiseGatherer.all()])
        return retrieved.response.clone()
    }

    private async retrieveSkinFromMojang(identity: string, identityType: IdentityKind, promiseGatherer: PromiseGatherer): Promise<SkinResponse> {
        const profile = await this.fetchMojangProfile(identity, identityType, promiseGatherer);
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
                    response: ResponseCacheService.cleanResponseForCache(textureResponse),
                    profile
                }
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return {
            response: new Response(STEVE_SKIN),
            profile: null
        }
    }

    private async fetchMojangProfile(identity: string, identityType: IdentityKind, promiseGatherer: PromiseGatherer): Promise<MojangProfile | null> {
        const doLookup = (id: string): Promise<Response> => {
            return fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${id}`, {
                cf: {
                    cacheEverything: true,
                    cacheTtl: MOJANG_API_TTL
                }
            })
        }

        let profilePromise: Promise<Response>
        switch (identityType) {
            case IdentityKind.Uuid:
                profilePromise = doLookup(identity)
                break
            case IdentityKind.Username:
                profilePromise = this.mapNameToUuid(identity, promiseGatherer)
                    .then((result) => {
                        if (!result) {
                            return new Response('', { status: 206 })
                        }
                        return doLookup(result);
                    });
                break;
        }

        const profileResponse = await profilePromise;
        if (profileResponse.status === 200) {
            const profile: MojangProfile = await profileResponse.json();
            promiseGatherer.push(this.saveUsernameCache(profile));
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
                cacheTtl: MOJANG_API_TTL
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

    async mapNameToUuid(name: string, promiseGatherer: PromiseGatherer): Promise<string | undefined> {
        const url = this.getNameCacheUrl(name.toLocaleLowerCase('en-US'));
        const cachedName = await caches.default.match(url);
        if (cachedName) {
            return cachedName.text();
        }

        const mojangResponse = await this.lookupUsernameFromMojang(name);
        if (mojangResponse) {
            promiseGatherer.push(this.saveUsernameCache(mojangResponse));
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