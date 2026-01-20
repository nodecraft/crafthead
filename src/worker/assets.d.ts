// Type declarations for Wrangler bundled assets

// Model and animation files bundled as text
declare module '*.blockymodel' {
	const content: string;
	export default content;
}

declare module '*.blockyanim' {
	const content: string;
	export default content;
}

// PNG files bundled as ArrayBuffer via Data type
declare module '*.png' {
	const content: ArrayBuffer;
	export default content;
}

// JSON files from CharacterCreator bundled as parsed objects
declare module '*/Cosmetics/CharacterCreator/*.json' {
	const content: unknown[];
	export default content;
}
