const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    mode: "none",
    target: "webworker",
    entry: "./worker/index.ts",
    output: {
        filename: "script.js",
        path: path.join(__dirname, 'worker', 'generated'),
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            }
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: __dirname,
            extraArgs: "--target bundler",
            outName: "crafthead"
        })
    ]
};