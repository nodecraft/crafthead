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

interface MojangTextureData {
    SKIN?: {
        url: string,
        metadata?: {
            model?: string;
        }
    };
    CAPE?: {
        url: string
    };
}

interface MojangTexturePropertyValue {
    textures: MojangTextureData;
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

    /**
     * Fetches a skin directly from the Mojang servers. Assumes the request has been normalized already.
     */
    private async retrieveSkinDirect(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        const rawUuid = fromHex(request.identity);
        if (uuidVersion(rawUuid) === 4) {
            const lookup = await this.mojangApi.fetchProfile(request.identity, gatherer);
            if (lookup.result) {
                let skinResponse = await MojangRequestService.fetchSkinTextureFromProfile(lookup.result);
                const buff = await skinResponse.texture.arrayBuffer();
                if (buff && buff.byteLength > 0) {
                    return new Response(buff, {
                        status: 200,
                        headers: {
                            'X-Crafthead-Profile-Cache-Hit': lookup.source,
                            'X-Crafthead-Skin-Model': skinResponse.slim ? 'slim' : 'default'
                        }
                    });
                }
            }
            return new Response(STEVE_SKIN, {
                status: 404,
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': 'not-found',
                    'X-Crafthead-Skin-Model': 'default'
                }
            });
        }

        return new Response(STEVE_SKIN, {
            status: 404,
            headers: {
                'X-Crafthead-Profile-Cache-Hit': 'offline-mode',
                'X-Crafthead-Skin-Model': 'default'
            }
        });
    }

    async retrieveSkin(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        if (request.identity === 'char' || request.identity === 'MHF_Steve') {
            // These are special-cased by Minotar.
            return new Response(STEVE_SKIN);
        }

        const normalized = await this.normalizeRequest(request, gatherer);
        const skin = await this.retrieveSkinDirect(normalized, gatherer);
        if (skin.status === 404) {
            // Offline mode ID (usually when we have a username and the username isn't valid)
            const rawUuid = fromHex(normalized.identity);
            if (Math.abs(javaHashCode(rawUuid)) % 2 == 0) {
                return new Response(STEVE_SKIN, {
                    headers: {
                        'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
                        'X-Crafthead-Skin-Model': 'default'
                    }
                });
            } else {
                return new Response(ALEX_SKIN, {
                    headers: {
                        'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
                        'X-Crafthead-Skin-Model': 'slim'
                    }
                });
            }
        }
        return skin;
    }

    private static async fetchSkinTextureFromProfile(profile: MojangProfile): Promise<{ texture: Response, slim?: boolean }> {
        if (profile.properties) {
            const texturesData = MojangRequestService.extractDataFromTexturesProperty(
                profile.properties.find(property => property.name === 'textures'));
            const textureUrl = texturesData?.url;

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
                return { texture: textureResponse, slim: texturesData?.slim };
            }
        }

        console.log("Invalid properties found! Falling back to Steve skin.")
        return { texture: new Response(STEVE_SKIN) };
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

    private static extractDataFromTexturesProperty(property: MojangProfileProperty | undefined): { url?: string, slim: boolean } | undefined {
        if (typeof property === 'undefined') {
            return undefined;
        }

        const rawJson = atob(property.value);
        const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);
        console.log("Raw textures property: ", property);

        const textures = decoded.textures;
        const slim = textures.SKIN?.metadata?.model === "slim";

        return { url: textures.SKIN?.url, slim };
    }
}