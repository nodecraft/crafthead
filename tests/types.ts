import type { Env } from '../worker/types';

declare module 'cloudflare:test' {
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type
	interface ProvidedEnv extends Env {}
}
