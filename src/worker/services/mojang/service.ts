import { ALEX_SKIN, EMPTY, STEVE_SKIN } from '../../data';
import { IdentityKind, RequestedKind, TextureKind } from '../../request';
import {
	fromHex,
	javaHashCode,
	offlinePlayerUuid,
	toHex,
	uuidVersion,
} from '../../util/uuid';

import type { MojangApiService, MojangProfile, MojangProfileProperty } from './api';
import type { CraftheadRequest } from '../../request';
import type { CacheComputeResult } from '../../util/cache-helper';

interface TextureResponse {
	texture: Response;
	model?: string;
}

interface MojangTextureData {
	SKIN: {
		url: string;
		metadata?: {
			model?: string;
		};
	};
	CAPE?: {
		url: string;
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
     */
	async normalizeRequest(request: CraftheadRequest): Promise<CraftheadRequest> {
		if (request.identityType === IdentityKind.Uuid || request.identityType === IdentityKind.TextureID) {
			return request;
		}

		const normalized: CraftheadRequest = { ...request, identityType: IdentityKind.Uuid };

		const profileLookup = await this.mojangApi.lookupUsername(request.identity);
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
	private async retrieveTextureDirect(request: CraftheadRequest, kind: TextureKind): Promise<TextureResponse> {
		if (request.identityType === IdentityKind.TextureID) {
			const textureResponse = await MojangRequestService.fetchTextureFromId(request.identity);
			return {
				texture: await MojangRequestService.constructTextureResponse(textureResponse, request),
			};
		}
		const rawUuid = fromHex(request.identity);
		if (uuidVersion(rawUuid) === 4) {
			const lookup = await this.mojangApi.fetchProfile(request.identity);
			if (lookup.result) {
				const textureResponse = await MojangRequestService.fetchTextureFromProfile(lookup.result, kind);
				if (textureResponse) {
					return {
						texture: await MojangRequestService.constructTextureResponse(textureResponse, request, lookup.source),
						model: textureResponse.model,
					};
				}
				return {
					texture: new Response(STEVE_SKIN, {
						status: 404,
						headers: {
							'X-Crafthead-Profile-Cache-Hit': 'not-found',
						},
					}),
				};
			}
			return {
				texture: new Response(STEVE_SKIN, {
					status: 404,
					headers: {
						'X-Crafthead-Profile-Cache-Hit': 'not-found',
					},
				}),
			};
		}

		return {
			texture: new Response(STEVE_SKIN, {
				status: 404,
				headers: {
					'X-Crafthead-Profile-Cache-Hit': 'offline-mode',
				},
			}),
		};
	}

	private static async constructTextureResponse(textureResponse: TextureResponse, request: CraftheadRequest, source?: string): Promise<Response> {
		const buff = await textureResponse.texture.arrayBuffer();
		if (buff && buff.byteLength > 0) {
			const headers = new Headers(textureResponse.texture.headers);
			if (!headers.has('X-Crafthead-Profile-Cache-Hit')) {
				headers.set('X-Crafthead-Profile-Cache-Hit', source || 'miss');
			}
			return new Response(buff, {
				status: 200,
				headers,
			});
		}
		return new Response(STEVE_SKIN, {
			status: 404,
			headers: {
				'X-Crafthead-Profile-Cache-Hit': 'not-found',
				'X-Crafthead-Skin-Model': 'default',
			},
		});
	}

	async retrieveSkin(request: CraftheadRequest): Promise<Response> {
		if (request.identity === 'char' || request.identity === 'MHF_Steve') {
			// These are special-cased by Minotar.
			return new Response(STEVE_SKIN);
		}

		const normalized = await this.normalizeRequest(request);
		const skin = await this.retrieveTextureDirect(normalized, TextureKind.SKIN);
		if (skin.texture.status === 404) {
			// Offline mode ID (usually when we have a username and the username isn't valid)
			const rawUuid = fromHex(normalized.identity);
			if (Math.abs(javaHashCode(rawUuid)) % 2 === 0) {
				return new Response(STEVE_SKIN, {
					headers: {
						'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
					},
				});
			}
			return new Response(ALEX_SKIN, {
				headers: {
					'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
				},
			});
		}
		if ([RequestedKind.Skin, RequestedKind.Body, RequestedKind.Bust].includes(normalized.requested)) {
			skin.texture.headers.set('X-Crafthead-Skin-Model', request.model || skin.model || 'default');
		}

		return skin.texture;
	}

	async retrieveCape(request: CraftheadRequest): Promise<Response> {
		const normalized = await this.normalizeRequest(request);
		const cape = await this.retrieveTextureDirect(normalized, TextureKind.CAPE);
		if (cape.texture.status === 404) {
			return new Response(EMPTY, {
				status: 404,
				headers: {
					'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
				},
			});
		}
		return cape.texture;
	}

	private static async fetchTextureFromProfile(profile: MojangProfile, type: TextureKind): Promise<TextureResponse | undefined> {
		if (profile.properties) {
			const texturesData = MojangRequestService.extractDataFromTexturesProperty(
				profile.properties.find(property => property.name === 'textures'),
			);
			const textureUrl = type === TextureKind.CAPE ? texturesData?.CAPE?.url : texturesData?.SKIN.url;

			if (textureUrl) {
				const textureResponse = await fetch(textureUrl, {
					cf: {
						cacheEverything: true,
						cacheTtl: 86400,
					},
					headers: {
						'User-Agent': 'Crafthead (+https://crafthead.net)',
					},
					signal: AbortSignal.timeout(5000),
				});
				if (!textureResponse.ok) {
					throw new Error(`Unable to retrieve texture from Mojang, http status ${textureResponse.status}`);
				}

				const response = new Response(textureResponse.body);
				const textureID = textureUrl.split('/').pop();
				if (textureID) {
					response.headers.set('X-Crafthead-Texture-ID', textureID);
				}
				//console.log('Successfully retrieved texture');
				return {
					texture: response,
					model: texturesData?.SKIN?.metadata?.model,
				};
			}
		}

		//console.log('Invalid properties found! Falling back to a default texture.');
		return undefined;
	}

	private static async fetchTextureFromId(id: string): Promise<TextureResponse> {
		const url = `https://textures.minecraft.net/texture/${id}`;
		return this.fetchTextureFromUrl(url);
	}

	private static async fetchTextureFromUrl(textureUrl: string): Promise<TextureResponse> {
		const textureResponse = await fetch(textureUrl, {
			cf: {
				cacheEverything: true,
				cacheTtl: 86400,
			},
			headers: {
				'User-Agent': 'Crafthead (+https://crafthead.net)',
			},
			signal: AbortSignal.timeout(5000),
		});
		if (!textureResponse.ok) {
			throw new Error(`Unable to retrieve texture from Mojang, http status ${textureResponse.status}`);
		}

		//console.log('Successfully retrieved texture');
		return { texture: textureResponse, model: 'default' };
	}

	async fetchProfile(request: CraftheadRequest): Promise<CacheComputeResult<MojangProfile | null>> {
		const normalized = await this.normalizeRequest(request);
		if (!normalized.identity || uuidVersion(fromHex(normalized.identity)) === 3) {
			return {
				result: null,
				source: 'mojang',
			};
		}
		return this.mojangApi.fetchProfile(normalized.identity);
	}

	private static extractDataFromTexturesProperty(property: MojangProfileProperty | undefined): MojangTextureData | undefined {
		if (property === undefined) {
			return undefined;
		}

		const rawJson = atob(property.value);
		const decoded: MojangTexturePropertyValue = JSON.parse(rawJson);

		return decoded.textures;
	}
}
