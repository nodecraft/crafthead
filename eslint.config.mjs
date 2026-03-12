import nodecraftConfig from '@nodecraft/eslint-config';

function withJsonIgnore(configs) {
	return configs.map((config) => {
		return {
			...config,
			ignores: [...(config.ignores || []), '**/*.json'],
		};
	});
}

export default [
	{ ignores: ['.wrangler/**', 'assets/**', 'dist/**', '**/target/**', 'pkg/**'] },
	...withJsonIgnore(nodecraftConfig.configs.node),
	...withJsonIgnore(nodecraftConfig.configs.typescript),
	...nodecraftConfig.configs.json,
	{
		rules: {
			'n/no-unsupported-features/node-builtins': 'off',
		},
	},
];
