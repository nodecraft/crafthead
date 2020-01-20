// This is a hack in order to get webpack to play nice with the included WebAssembly module.
// See https://github.com/rustwasm/wasm-bindgen/issues/700 for more details.
export async function getRenderer(): Promise<{
    get_minecraft_head(skin_image: any, size: number): any;
    blake3(skin_image: any): string;
}> {
    return new Promise((resolve, reject) => {
        require.ensure([], function () {
            const renderer = require("../pkg/crafthead")
            return resolve(renderer);
        })
    })
}