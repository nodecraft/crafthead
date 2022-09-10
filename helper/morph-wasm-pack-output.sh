#!/bin/sh

# The starting point for getting Wasm working is to produce a Node.js-style module, then
# strip out all the Node stuff. We will use the standard Unix editor for this because
# I have a penchant for punishment.
ed -s pkg/mcavatar.js <<EOF
4c
import wasmModule from './mcavatar_bg.wasm';
.
182,186d
w
EOF