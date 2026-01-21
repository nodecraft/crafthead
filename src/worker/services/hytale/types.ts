/**
 * Hytale Cosmetic Type Definitions
 *
 * Types for cosmetic definitions from CharacterCreator JSON files
 * and resolved cosmetics with asset paths.
 */

import type { HytaleSkin } from './api';

/**
 * Cosmetic slot names that map to HytaleSkin fields
 */
export type CosmeticSlot = keyof HytaleSkin;

/**
 * Maps slot names to their CharacterCreator JSON file names
 */
export const SLOT_TO_FILE: Record<CosmeticSlot, string> = {
	bodyCharacteristic: 'BodyCharacteristics',
	underwear: 'Underwear',
	face: 'Faces',
	ears: 'Ears',
	mouth: 'Mouths',
	haircut: 'Haircuts',
	facialHair: 'FacialHair',
	eyebrows: 'Eyebrows',
	eyes: 'Eyes',
	pants: 'Pants',
	overpants: 'Overpants',
	undertop: 'Undertops',
	overtop: 'Overtops',
	shoes: 'Shoes',
	headAccessory: 'HeadAccessory',
	faceAccessory: 'FaceAccessory',
	earAccessory: 'EarAccessory',
	skinFeature: 'SkinFeatures',
	gloves: 'Gloves',
	cape: 'Capes',
};

/**
 * Variant definition within a cosmetic
 */
export interface CosmeticVariant {
	Model?: string;
	GreyscaleTexture?: string;
	Texture?: string;
	NameKey?: string;
	Icon?: string;
	BaseColor?: string[];
}

/**
 * Texture definition for non-gradient cosmetics
 */
export interface CosmeticTexture {
	Texture: string;
	BaseColor: string[];
}

/**
 * Raw cosmetic definition from CharacterCreator JSON files
 */
export interface CosmeticDefinition {
	Id: string;
	Name?: string;
	Model?: string;
	GreyscaleTexture?: string;
	GradientSet?: string;
	HairType?: string;
	RequiresGenericHaircut?: boolean;
	IsDefaultAsset?: boolean;
	Variants?: Record<string, CosmeticVariant>;
	Textures?: Record<string, CosmeticTexture>;
	Entitlements?: string[];
}

/**
 * Gradient color definition
 */
export interface GradientColor {
	BaseColor: string[];
	Texture: string;
}

/**
 * Gradient set definition from GradientSets.json
 */
export interface GradientSetDefinition {
	Id: string;
	Gradients: Record<string, GradientColor>;
}

/**
 * Resolved cosmetic with all paths needed for rendering
 */
export interface ResolvedCosmetic {
	slot: CosmeticSlot;
	id: string;
	modelPath: string | null;
	texturePath: string | null;
	gradientSetId: string | null;
	colorId: string | null;
	gradientTexturePath: string | null;
	baseColor: string[] | null;
	variant?: string;
}

/**
 * Result of resolving a full skin configuration
 */
export interface ResolvedSkin {
	cosmetics: ResolvedCosmetic[];
	skinTone: ResolvedCosmetic | null;
}
