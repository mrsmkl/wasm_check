#!/bin/sh

cargo build

cd test

emcc -o test.js test.c

node ~/emscripten-module-wrapper/prepare.js test.js --memory-size 20 --run

# copy globals.wasm here

../target/debug/wasm_check

wasm -m -table-size 20 -stack-size 20 -memory-size 20 -wasm output.wasm -memory-offset 10240

