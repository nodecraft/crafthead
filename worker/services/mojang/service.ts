/// <reference path="./mojang.d.ts">

import PromiseGatherer from '../../promise_gather';
import { IdentityKind, CraftheadRequest } from '../../request';
import { ALEX_SKIN, STEVE_SKIN } from '../../data';
import { MojangApiService, MojangProfile, MojangProfileProperty } from "./api";
import { CacheComputeResult } from '../../util/cache-helper';
import { fromHex, javaHashCode, offlinePlayerUuid, toHex, uuidVersion } from '../../util/uuid';

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
    async normalizeRequest(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<CraftheadRequest> {
        if (request.identityType === IdentityKind.Uuid) {
            return request;
        }

        const normalized: CraftheadRequest = { ...request, identityType: IdentityKind.Uuid };

        const profileLookup = await this.mojangApi.lookupUsername(request.identity, gatherer);
        if (profileLookup) {
            normalized.identity = profileLookup.id;
        } else {
            // The lookup failed.
            normalized.identity = toHex(await offlinePlayerUuid(request.identity));
        }
        return normalized;
    }

    async retrieveSkin(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        if (request.identity === 'char' || request.identity === 'MHF_Steve') {
            // These are special-cased by Minotar.
            return new Response(STEVE_SKIN);
        }

        const normalized = await this.normalizeRequest(request, gatherer);
        if (!normalized.identity) {
            // TODO: Can't figure out why this is inexplicitly undefined(!)
            return new Response(STEVE_SKIN, {
                headers: {
                    'X-Crafthead-Skin-Cache-Hit': 'unknown'
                }
            });
        }
        const rawUuid = fromHex(normalized.identity);
        if (uuidVersion(rawUuid) === 4) {
            // See if the player has a skin.
            const lookup = await this.mojangApi.fetchProfile(normalized.identity, gatherer);
            let buff: ArrayBuffer | null = null;
            if (lookup.result !== null) {
                let skinResponse = await this.fetchSkinTextureFromProfile(lookup.result);
                buff = await skinResponse.arrayBuffer();
            }
            
            if (buff && buff.byteLength > 0) {
                return new Response(buff, {
                    status: 200,
                    headers: {
                        'X-Crafthead-Profile-Cache-Hit': lookup.source
                    }
                });
            }
        }

        if (Math.abs(javaHashCode(rawUuid)) % 2 == 0) {
            return new Response(STEVE_SKIN, {
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': 'invalid-profile'
                }
            });
        } else {
            return new Response(ALEX_SKIN, {
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': 'invalid-profile'
                }
            });
        }
    }

    private async fetchSkinTextureFromProfile(profile: MojangProfile): Promise<Response> {
        if (profile.properties) {
            const textureUrl = MojangRequestService.extractUrlFromTexturesProperty(
                profile.properties.find(property => property.name === 'textures'));
            if (textureUrl) {
                const textureResponse = await fetch(textureUrl, {
                    cf: {
                        cacheEverything: true,
                        cacheTtl: 86400
                    },
                    headers: {
                        'User-Agent': 'Crafthead (+https://crafthead.net)'
                    }
                });
                if (!textureResponse.ok) {
                    throw new Error(`Unable to retrieve skin texture from Mojang, http status ${textureResponse.status}`);
                }

                console.log("Successfully retrieved skin texture");
                return textureResponse;
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return new Response(STEVE_SKIN);
    }

    async fetchProfile(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<CacheComputeResult<MojangProfile | null>> {
        const normalized = await this.normalizeRequest(request, gatherer);
        if (!normalized.identity || uuidVersion(fromHex(normalized.identity)) === 3) {
            return {
                result: null,
                source: 'mojang'
            };
        }
        return this.mojangApi.fetchProfile(normalized.identity, gatherer);
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