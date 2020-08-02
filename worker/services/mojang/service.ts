/// <reference path="./mojang.d.ts">

import PromiseGatherer from '../../promise_gather';
import {IdentityKind, MineheadRequest} from '../../request';
import {STEVE_SKIN} from '../../data';
import {ArrayBufferCloudflareResponseMapper, CloudflareCacheService} from '../cache/cloudflare';
import ResponseCacheService from '../cache/response_helper';
import {MojangApiService, MojangProfile, MojangProfileProperty} from "./api";
import NoopCacheService from "../cache/noop";

const FAKE_MHF_STEVE_UUID = '!112548de8d0c42a78745aabac5a64ebb'

export interface SkinResponse {
    response: Response;
    profile: MojangProfile | null;
}

interface MojangTextureUrls {
    SKIN: { url: string } | undefined;
    CAPE: { url: string } | undefined;
}

interface MojangTexturePropertyValue {
    textures: MojangTextureUrls;
}

export default class MojangRequestService {
    private skinCache: CacheService<Response>;
    private mojangApi: MojangApiService;

    constructor(mojangApi: MojangApiService) {
        this.skinCache = new ResponseCacheService(
            new NoopCacheService(),
            new CloudflareCacheService('skin', new ArrayBufferCloudflareResponseMapper())
        );
        this.mojangApi = mojangApi;
    }

    /**
     * Normalizes the incoming request, such that we only work with UUIDs. A new request bearing an UUID is returned.
     * @param request the incoming request
     * @param gatherer any promise gatherer
     */
    async normalizeRequest(request: MineheadRequest, gatherer: PromiseGatherer): Promise<MineheadRequest> {
        if (request.identityType === IdentityKind.Uuid) {
            return request;
        }

        const normalized: MineheadRequest = Object.assign({}, request);
        normalized.identityType = IdentityKind.Uuid;

        const nameUrlCacheRequest = new Request(`https://crafthead.net/username-lookup/${request.identity.toLocaleLowerCase('en-US')}`);
        const cachedUuid = await caches.default.match(nameUrlCacheRequest);
        if (typeof cachedUuid === 'undefined') {
            const profileLookup = await this.mojangApi.lookupUsername([request.identity]);
            if (profileLookup) {
                normalized.identity = profileLookup.id
            } else {
                // The lookup failed.
                normalized.identity = FAKE_MHF_STEVE_UUID;
            }

            gatherer.push(caches.default.put(nameUrlCacheRequest, new Response(normalized.identity, {
                headers: {
                    "Cache-Control": "max-age=86400"
                }
            })));
        } else {
            normalized.identity = await cachedUuid.text()
        }
        return normalized;
    }

    async retrieveSkin(request: MineheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        if (request.identity === 'char' || request.identity === 'MHF_Steve') {
            // These are special-cased by Minotar.
            return new Response(STEVE_SKIN);
        }

        const lowercaseId = request.identity.toLocaleLowerCase('en-US');

        // See if we already have the skin cached already. This should is a very cheap check.
        const cachedSkin = await this.skinCache.find(lowercaseId);
        if (cachedSkin) {
            return cachedSkin;
        }

        // We don't - so let's fetch the user's UUID and use that instead. Note that normalization won't do anything
        // if the request is by UUID anyway, so there is no performance penalty paid in this case.
        const normalized = await this.normalizeRequest(request, gatherer);

        const retrieved = await this.retrieveSkinFromMojang(normalized.identity);
        gatherer.push(this.skinCache.put(lowercaseId, retrieved.response.clone()));
        if (normalized.identity !== lowercaseId) {
            gatherer.push(this.skinCache.put(normalized.identity, retrieved.response.clone()));
        }
        return retrieved.response.clone();
    }

    private async retrieveSkinFromMojang(identity: string): Promise<SkinResponse> {
        const profile = await this.mojangApi.fetchProfile(identity);
        if (profile?.properties) {
            const textureUrl = MojangRequestService.extractUrlFromTexturesProperty(
                profile.properties.find(property => property.name === 'textures'));
            if (textureUrl) {
                const textureResponse = await fetch(textureUrl);
                if (!textureResponse.ok) {
                    throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`);
                }

                console.log("Successfully retrieved skin texture");
                return {
                    response: ResponseCacheService.cleanResponseForCache(textureResponse),
                    profile
                };
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return {
            response: new Response(STEVE_SKIN),
            profile: null
        }
    }

    async fetchProfile(request: MineheadRequest, gatherer: PromiseGatherer): Promise<MojangProfile | null> {
        if (request.identityType === IdentityKind.Uuid) {
            return this.mojangApi.fetchProfile(request.identity);
        } else {
            const normalized = await this.normalizeRequest(request, gatherer);
            return this.mojangApi.fetchProfile(normalized.identity);
        }
    }

    private static extractUrlFromTexturesProperty(property: MojangProfileProperty | undefined): string | undefined {
        if (typeof property === 'undefined') {
            return undefined;
        }

        const rawJson = atob(property.value);
        const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);
        console.log("Raw textures property: ", property);

        const textures = decoded.textures;
        return textures.SKIN?.url;
    }
}