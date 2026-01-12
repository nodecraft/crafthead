import { describe, expect, it } from 'vitest';

import mcavatar from '../pkg/mcavatar';

describe('wasm', async () => {
	it('should render a skin', async () => {
		const skinRes = await fetch('https://textures.minecraft.net/texture/9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');
		const skinArrayBuffer = await skinRes.arrayBuffer();
		const skinBuf = new Uint8Array(skinArrayBuffer);
		const image = mcavatar.get_rendered_image(skinBuf, 64, 'avatar', false, false, 'minecraft');
		expect(image).toBeDefined();
	});

	it('should throw with bad input', async () => {
		const skinBuf = new Uint8Array(0);
		expect(() => mcavatar.get_rendered_image(skinBuf, 64, 'avatar', false, false, 'minecraft')).toThrowError();
	});
});
