/// <reference path="./mojang.d.ts">

import PromiseGatherer from '../../promise_gather';
import {IdentityKind, MineheadRequest} from '../../request';
import {STEVE_SKIN} from '../../data';
import {MojangApiService, MojangProfile, MojangProfileLookupResult, MojangProfileProperty} from "./api";

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
    private mojangApi: MojangApiService;

    constructor(mojangApi: MojangApiService) {
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

        const profileLookup = await this.mojangApi.lookupUsername(request.identity, gatherer);
        if (profileLookup) {
            normalized.identity = profileLookup.id;
        } else {
            // The lookup failed.
            normalized.identity = FAKE_MHF_STEVE_UUID;
        }
        return normalized;
    }

    async retrieveSkin(request: MineheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        if (request.identity === 'char' || request.identity === 'MHF_Steve' || request.identity === FAKE_MHF_STEVE_UUID) {
            // These are special-cased by Minotar.
            return new Response(STEVE_SKIN);
        }

        const profile = await this.fetchProfile(request, gatherer);
        if (profile === null) {
            return new Response(STEVE_SKIN);
        }

        return this.fetchSkinTextureFromProfile(profile);
    }

    private async fetchSkinTextureFromProfile(lookup: MojangProfileLookupResult): Promise<Response> {
        if (lookup.profile?.properties) {
            const profile = lookup.profile
            const textureUrl = MojangRequestService.extractUrlFromTexturesProperty(
                profile.properties.find(property => property.name === 'textures'));
            if (textureUrl) {
                const textureResponse = await fetch(textureUrl, {
                    cf: {
                        cacheTtl: 86400
                    }
                });
                if (!textureResponse.ok) {
                    throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`);
                }

                console.log("Successfully retrieved skin texture");
                const cleanedHeaders = new Headers(textureResponse.headers)
                for (const [key] of textureResponse.headers.entries()) {
                    if (key !== 'content-type' && key !== 'content-length') {
                        if (key === 'x-amz-cf-id') {
                            cleanedHeaders.set('X-Minehead-Profile-Cache-Hit', lookup.source);
                        }
                        cleanedHeaders.delete(key);
                    }
                }
                return new Response(textureResponse.body, {
                    status: textureResponse.status,
                    headers: cleanedHeaders
                });
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return new Response(STEVE_SKIN);
    }

    async fetchProfile(request: MineheadRequest, gatherer: PromiseGatherer): Promise<MojangProfileLookupResult> {
        if (request.identityType === IdentityKind.Uuid) {
            return this.mojangApi.fetchProfile(request.identity, gatherer);
        } else {
            const normalized = await this.normalizeRequest(request, gatherer);
            if (normalized.identity === FAKE_MHF_STEVE_UUID) {
                return {
                    profile: null,
                    source: 'mojang'
                };
            }
            return this.mojangApi.fetchProfile(normalized.identity, gatherer);
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