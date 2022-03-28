/// <reference path="./mojang.d.ts">

import PromiseGatherer from '../../promise_gather';
import { IdentityKind, CraftheadRequest, TextureKind } from '../../request';
import { ALEX_SKIN, STEVE_SKIN } from '../../data';
import { MojangApiService, MojangProfile, MojangProfileProperty } from "./api";
import { CacheComputeResult } from '../../util/cache-helper';
import { fromHex, javaHashCode, offlinePlayerUuid, toHex, uuidVersion } from '../../util/uuid';

interface TextureResponse {
    texture: Response,
    model?: string;
}

interface MojangTextureData {
    SKIN: {
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
        if (request.identityType === IdentityKind.Uuid || request.identityType == IdentityKind.TextureID) {
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
     * Fetches a texture directly from the Mojang servers. Assumes the request has been normalized already.
     */
    private async retrieveTextureDirect(request: CraftheadRequest, gatherer: PromiseGatherer, kind: TextureKind): Promise<Response> {
        if (request.identityType == IdentityKind.TextureID) {
            const textureResponse = await MojangRequestService.fetchTextureFromId(request.identity);
            return MojangRequestService.constructTextureResponse(textureResponse, request);
        }

        const rawUuid = fromHex(request.identity);
        if (uuidVersion(rawUuid) === 4) {
            const lookup = await this.mojangApi.fetchProfile(request.identity, gatherer);
            if (lookup.result) {
                let textureResponse = await MojangRequestService.fetchTextureFromProfile(lookup.result, kind);
                if (textureResponse) {
                    return MojangRequestService.constructTextureResponse(textureResponse, request, lookup.source)
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

    private static async constructTextureResponse(textureResponse: TextureResponse, request: CraftheadRequest, source?: string): Promise<Response> {
        const buff = await textureResponse.texture.arrayBuffer();
        if (buff && buff.byteLength > 0) {
            return new Response(buff, {
                status: 200,
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': source || 'miss',
                    'X-Crafthead-Skin-Model': request.model || textureResponse.model || 'default'
                }
            });
        } else {
            return new Response(STEVE_SKIN, {
                status: 404,
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': 'not-found',
                    'X-Crafthead-Skin-Model': 'default'
                }
            });
        }
    }

    async retrieveSkin(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        if (request.identity === 'char' || request.identity === 'MHF_Steve') {
            // These are special-cased by Minotar.
            return new Response(STEVE_SKIN);
        }

        const normalized = await this.normalizeRequest(request, gatherer);
        const skin = await this.retrieveTextureDirect(normalized, gatherer, TextureKind.SKIN);
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

    async retrieveCape(request: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
        const normalized = await this.normalizeRequest(request, gatherer);
        return this.retrieveTextureDirect(normalized, gatherer, TextureKind.CAPE);
    }

    private static async fetchTextureFromProfile(profile: MojangProfile, type: TextureKind): Promise<TextureResponse | undefined> {
        if (profile.properties) {
            const texturesData = MojangRequestService.extractDataFromTexturesProperty(
                profile.properties.find(property => property.name === 'textures'));
            const textureUrl = type === TextureKind.CAPE ? texturesData?.CAPE?.url : texturesData?.SKIN.url;

            if (textureUrl) {
                return this.fetchTextureFromUrl(textureUrl);
            }
        }

        console.log("Invalid properties found! Falling back to a default texture.")
        return undefined;
    }

    private static async fetchTextureFromId(id: string): Promise<TextureResponse> {
        const url = `https://textures.minecraft.net/texture/${id}`
        return this.fetchTextureFromUrl(url);
    }

    private static async fetchTextureFromUrl(textureUrl: string): Promise<TextureResponse> {
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
            throw new Error(`Unable to retrieve texture from Mojang, http status ${textureResponse.status}`);
        }

        console.log("Successfully retrieved texture");
        return { texture: textureResponse, model: "default" };
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

    private static extractDataFromTexturesProperty(property: MojangProfileProperty | undefined): MojangTextureData | undefined {
        if (typeof property === 'undefined') {
            return undefined;
        }

        const rawJson = atob(property.value);
        const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);
        console.log("Raw textures property: ", property);

        return decoded.textures;
    }
}