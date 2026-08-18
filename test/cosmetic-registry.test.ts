import { describe, expect, it } from 'vitest';

import { resolveCosmetic, resolvePart } from '../src/worker/services/hytale/cosmetic-registry';

// Backed by the real bundled CharacterCreator JSON: Shoes.json renames QuiltedBoots ->
// Boots_Voyager (part level), Capes.json renames Cape_Trimmed -> Cape_Royal_Emissary's
// Neck_Piece variant (variant level).
describe('cosmetic fallback resolution', () => {
	it('resolves a renamed part with no forced variant', async () => {
		const resolved = await resolvePart('shoes', 'QuiltedBoots');
		expect(resolved?.definition.Id).toBe('Boots_Voyager');
		expect(resolved?.forcedVariant).toBeUndefined();
	});

	it('resolves a renamed variant to its new part plus the forced variant', async () => {
		const resolved = await resolvePart('cape', 'Cape_Trimmed');
		expect(resolved?.definition.Id).toBe('Cape_Royal_Emissary');
		expect(resolved?.forcedVariant).toBe('Neck_Piece');
	});

	it('returns null for an unknown id', async () => {
		expect(await resolvePart('shoes', 'DoesNotExist')).toBeNull();
	});

	it('selects the renamed part assets and carries colour through resolveCosmetic', async () => {
		const cosmetic = await resolveCosmetic('shoes', 'QuiltedBoots.Black');
		expect(cosmetic).not.toBeNull();
		// Assets fetched for WASM must point at the new part, with the saved colour kept.
		expect(cosmetic?.id).toBe('Boots_Voyager');
		expect(cosmetic?.modelPath).toContain('Cosmetics/Shoes/QuiltedBoots.blockymodel');
		expect(cosmetic?.colorId).toBe('Black');
	});
});
