// This is a hack in order to get webpack to play nice with the included WebAssembly module.
// See https://github.com/rustwasm/wasm-bindgen/issues/700 for more details.
export async function getWASMModule(): Promise<{
    get_rendered_image(skin_image: any, size: number, type: string, armored: boolean, slim: boolean): any;
    xxhash(value: string, seed: number): bigint;
}> {
    return new Promise((resolve, reject) => {
        // We intentionally ignore the erros here. We know for a fact we are using webpack targeting CommonJS,
        // so the error is spirous.
        
        // @ts-ignore
        require.ensure([], function () {
            // @ts-ignore
            const renderer = require("../pkg/crafthead")
            return resolve(renderer);
        })
    })
}