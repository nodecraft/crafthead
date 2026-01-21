/**
 * Hytale Cosmetic Registry
 *
 * Loads and indexes CharacterCreator JSON files for cosmetic lookups.
 * Provides methods to resolve cosmetic IDs to their definitions.
 *
 * NOTE: These JSON files are bundled with the worker because they contain only
 * metadata (file paths, IDs, color definitions) and are not copyright-infringing
 * game assets. The actual model/texture/animation files are loaded from R2.
 */

import { SLOT_TO_FILE } from './types';
import bodyCharacteristicsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/BodyCharacteristics.json';
import capesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Capes.json';
import earAccessoryJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/EarAccessory.json';
import earsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Ears.json';
import eyebrowsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Eyebrows.json';
import eyesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Eyes.json';
import faceAccessoryJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/FaceAccessory.json';
import facesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Faces.json';
import facialHairJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/FacialHair.json';
import glovesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Gloves.json';
import gradientSetsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/GradientSets.json';
import haircutsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Haircuts.json';
import headAccessoryJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/HeadAccessory.json';
import mouthsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Mouths.json';
import overpantsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Overpants.json';
import overtopsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Overtops.json';
import pantsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Pants.json';
import shoesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Shoes.json';
import skinFeaturesJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/SkinFeatures.json';
import undertopsJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Undertops.json';
import underwearJson from '../../../../assets/hytale/Cosmetics/CharacterCreator/Underwear.json';

import type { HytaleSkin } from './api';
import type {
	CosmeticDefinition,
	CosmeticSlot,
	GradientColor,
	GradientSetDefinition,
	ResolvedCosmetic,
	ResolvedSkin,
} from './types';

// Helper to parse JSON imports (Wrangler imports text, not objects)
function parseJson<T>(json: unknown): T {
	if (typeof json === 'string') {
		return JSON.parse(json) as T;
	}
	return json as T;
}

// Type the imported JSON arrays
type SlotDefinitions = Record<string, CosmeticDefinition[]>;

/**
 * Map of slot file names to their loaded definitions
 */
const DEFINITION_FILES: SlotDefinitions = {
	BodyCharacteristics: parseJson<CosmeticDefinition[]>(bodyCharacteristicsJson),
	Capes: parseJson<CosmeticDefinition[]>(capesJson),
	EarAccessory: parseJson<CosmeticDefinition[]>(earAccessoryJson),
	Ears: parseJson<CosmeticDefinition[]>(earsJson),
	Eyebrows: parseJson<CosmeticDefinition[]>(eyebrowsJson),
	Eyes: parseJson<CosmeticDefinition[]>(eyesJson),
	FaceAccessory: parseJson<CosmeticDefinition[]>(faceAccessoryJson),
	Faces: parseJson<CosmeticDefinition[]>(facesJson),
	FacialHair: parseJson<CosmeticDefinition[]>(facialHairJson),
	Gloves: parseJson<CosmeticDefinition[]>(glovesJson),
	Haircuts: parseJson<CosmeticDefinition[]>(haircutsJson),
	HeadAccessory: parseJson<CosmeticDefinition[]>(headAccessoryJson),
	Mouths: parseJson<CosmeticDefinition[]>(mouthsJson),
	Overpants: parseJson<CosmeticDefinition[]>(overpantsJson),
	Overtops: parseJson<CosmeticDefinition[]>(overtopsJson),
	Pants: parseJson<CosmeticDefinition[]>(pantsJson),
	Shoes: parseJson<CosmeticDefinition[]>(shoesJson),
	SkinFeatures: parseJson<CosmeticDefinition[]>(skinFeaturesJson),
	Undertops: parseJson<CosmeticDefinition[]>(undertopsJson),
	Underwear: parseJson<CosmeticDefinition[]>(underwearJson),
};

/**
 * Gradient sets indexed by ID
 */
const GRADIENT_SETS: Map<string, GradientSetDefinition> = new Map(
	parseJson<GradientSetDefinition[]>(gradientSetsJson).map(set => [set.Id, set]),
);

/**
 * Pre-indexed definitions by slot and ID for fast lookup
 */
const INDEXED_DEFINITIONS: Map<string, Map<string, CosmeticDefinition>> = new Map();

// Build index on load
for (const [fileName, definitions] of Object.entries(DEFINITION_FILES)) {
	const indexedById = new Map<string, CosmeticDefinition>();
	for (const def of definitions) {
		if (def.Id) {
			indexedById.set(def.Id, def);
		}
	}
	INDEXED_DEFINITIONS.set(fileName, indexedById);
}

/**
 * Look up a cosmetic definition by slot and ID
 */
export function getDefinition(slot: CosmeticSlot, id: string): CosmeticDefinition | null {
	const fileName = SLOT_TO_FILE[slot];
	if (!fileName) {
		return null;
	}
	const slotIndex = INDEXED_DEFINITIONS.get(fileName);
	if (!slotIndex) {
		return null;
	}
	return slotIndex.get(id) ?? null;
}

/**
 * Look up a gradient color by set ID and color ID
 */
export function getGradient(setId: string, colorId: string): GradientColor | null {
	const gradientSet = GRADIENT_SETS.get(setId);
	if (!gradientSet) {
		return null;
	}
	return gradientSet.Gradients[colorId] ?? null;
}

/**
 * Parse a skin value to extract ID and optional color/variant
 *
 * Skin values can be in formats like:
 * - "Default" (just ID)
 * - "WavyShort.BrownDark" (ID with color)
 * - "Cape_Wasteland_Marauder.Red.NoNeck" (ID with color and variant)
 */
export function parseSkinValue(value: string): { id: string; color?: string; variant?: string; } {
	const parts = value.split('.');
	if (parts.length === 1) {
		return { id: parts[0] };
	} else if (parts.length === 2) {
		// Could be id.color or id.variant - need to check definition
		return { id: parts[0], color: parts[1] };
	} else if (parts.length >= 3) {
		return { id: parts[0], color: parts[1], variant: parts.slice(2).join('.') };
	}
	return { id: value };
}

/**
 * Resolve a single cosmetic slot to its full definition and asset paths
 */
export function resolveCosmetic(
	slot: CosmeticSlot,
	value: string | null,
): ResolvedCosmetic | null {
	if (!value) {
		return null;
	}

	const parsed = parseSkinValue(value);
	const definition = getDefinition(slot, parsed.id);

	if (!definition) {
		console.warn(`Cosmetic definition not found: ${slot}/${parsed.id}`);
		return null;
	}

	// Determine model and texture paths
	let modelPath: string | null = null;
	let texturePath: string | null = null;
	let baseColor: string[] | null = null;
	let gradientSetId: string | null = definition.GradientSet ?? null;

	// Check for variant
	if (parsed.variant && definition.Variants) {
		const variant = definition.Variants[parsed.variant];
		if (variant) {
			modelPath = variant.Model ?? definition.Model ?? null;
			texturePath = variant.GreyscaleTexture ?? variant.Texture ?? null;
			if (variant.BaseColor) {
				baseColor = variant.BaseColor;
			}
		}
	} else if (definition.Textures) {
		const textureKey = parsed.color ?? Object.keys(definition.Textures)[0];
		const texture = definition.Textures[textureKey];
		if (texture) {
			texturePath = texture.Texture;
			baseColor = texture.BaseColor;
			gradientSetId = null; // No gradient for pre-colored textures
		}
		modelPath = definition.Model ?? null;
	} else {
		modelPath = definition.Model ?? null;
		texturePath = definition.GreyscaleTexture ?? null;
	}

	// Resolve gradient if applicable
	let gradientTexturePath: string | null = null;
	let colorId: string | null = null;
	if (gradientSetId && parsed.color) {
		colorId = parsed.color;
		const gradient = getGradient(gradientSetId, parsed.color);
		if (gradient) {
			gradientTexturePath = gradient.Texture;
			baseColor = gradient.BaseColor;
		}
	}

	return {
		slot,
		id: parsed.id,
		modelPath: modelPath ? `Common/${modelPath}` : null,
		texturePath: texturePath ? `Common/${texturePath}` : null,
		gradientSetId,
		colorId,
		gradientTexturePath: gradientTexturePath ? `Common/${gradientTexturePath}` : null,
		baseColor,
		variant: parsed.variant,
	};
}

/**
 * Resolve a full skin configuration to all cosmetics and their assets
 */
export function resolveSkin(skin: HytaleSkin): ResolvedSkin {
	const cosmetics: ResolvedCosmetic[] = [];

	// Iterate over all slots in the skin
	const slots: CosmeticSlot[] = [
		'bodyCharacteristic',
		'underwear',
		'face',
		'ears',
		'mouth',
		'haircut',
		'facialHair',
		'eyebrows',
		'eyes',
		'pants',
		'overpants',
		'undertop',
		'overtop',
		'shoes',
		'headAccessory',
		'faceAccessory',
		'earAccessory',
		'skinFeature',
		'gloves',
		'cape',
	];

	for (const slot of slots) {
		const value = skin[slot];
		const resolved = resolveCosmetic(slot, value);
		if (resolved) {
			cosmetics.push(resolved);
		}
	}

	const bodyCharacteristicIndex = cosmetics.findIndex(cos => cos.slot === 'bodyCharacteristic');
	let skinTone: ResolvedCosmetic | null = null;

	if (bodyCharacteristicIndex !== -1) {
		skinTone = cosmetics[bodyCharacteristicIndex];
		cosmetics.splice(bodyCharacteristicIndex, 1);
	}

	// Propagate skin tone to other cosmetics that need it (e.g. Face, Ears)
	// These cosmetics often have "GradientSet": "Skin" but no explicit color in the profile
	if (skinTone && skinTone.gradientTexturePath) {
		for (const cosmetic of cosmetics) {
			const needsSkinTone = !cosmetic.gradientTexturePath && (
				cosmetic.gradientSetId === 'Skin' ||
				(skinTone.gradientSetId && cosmetic.gradientSetId === skinTone.gradientSetId)
			);

			if (needsSkinTone) {
				cosmetic.gradientTexturePath = skinTone.gradientTexturePath;
				cosmetic.baseColor = skinTone.baseColor;
				cosmetic.colorId = skinTone.colorId;
				// cosmetic.gradientSetId is already set matching the definition
			}
		}
	}

	return {
		cosmetics,
		skinTone,
	};
}

/**
 * Get all unique asset paths needed to render a resolved skin
 */
export function getRequiredAssetPaths(resolvedSkin: ResolvedSkin): {
	models: string[];
	textures: string[];
	gradients: string[];
} {
	const models = new Set<string>();
	const textures = new Set<string>();
	const gradients = new Set<string>();

	for (const cosmetic of resolvedSkin.cosmetics) {
		if (cosmetic.modelPath) {
			models.add(cosmetic.modelPath);
		}
		if (cosmetic.texturePath) {
			textures.add(cosmetic.texturePath);
		}
		if (cosmetic.gradientTexturePath) {
			gradients.add(cosmetic.gradientTexturePath);
		}
	}

	if (resolvedSkin.skinTone?.gradientTexturePath) {
		gradients.add(resolvedSkin.skinTone.gradientTexturePath);
	}

	return {
		models: [...models],
		textures: [...textures],
		gradients: [...gradients],
	};
}
