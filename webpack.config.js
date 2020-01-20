const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    target: "webworker",
    entry: "./worker/index.js",
    output: {
        filename: "script.js",
        path: path.join(__dirname, 'worker', 'generated')
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: __dirname,
            extraArgs: "--typescript --target bundler",
            outName: "crafthead"
        }),
    ]
};