FROM ubuntu:18.04
MAINTAINER Sami Mäkelä

SHELL ["/bin/bash", "-c"]

RUN apt-get update \
 && apt-get install -y git cmake ninja-build g++ python wget ocaml opam libzarith-ocaml-dev m4 pkg-config zlib1g-dev apache2 psmisc sudo mongodb curl tmux nano libffi-dev \
 && opam init -y

RUN git clone https://github.com/juj/emsdk \
 && cd emsdk \
 && ./emsdk update-tags \
 && ./emsdk install sdk-1.37.36-64bit \
 && ./emsdk activate sdk-1.37.36-64bit \
 && ./emsdk install  binaryen-tag-1.37.36-64bit \
 && ./emsdk activate binaryen-tag-1.37.36-64bit

RUN cd bin \
 && wget https://github.com/ethereum/solidity/releases/download/v0.4.23/solc-static-linux \
 && mv solc-static-linux solc \
 && chmod 744 solc

RUN eval `opam config env` \
 && opam install cryptokit yojson ctypes ctypes-foreign -y \
 && git clone https://github.com/TrueBitFoundation/ocaml-offchain \
 && cd ocaml-offchain/interpreter \
 && git checkout gas-i64 \
 && make

RUN git clone https://github.com/TrueBitFoundation/emscripten-module-wrapper \
 && source /emsdk/emsdk_env.sh \
 && cd emscripten-module-wrapper \
 && git checkout fixed \
 && npm install

RUN wget -O rustup.sh https://sh.rustup.rs \
 && sh rustup.sh -y \
 && source $HOME/.cargo/env \
 && rustup toolchain add stable \
 && rustup target add wasm32-unknown-emscripten --toolchain stable

RUN git clone https://github.com/mrsmkl/parity-wasm

RUN git clone https://github.com/mrsmkl/wasm_check \
 && source /emsdk/emsdk_env.sh \
 && source $HOME/.cargo/env \
 && cd wasm_check \
 && cargo build --target wasm32-unknown-emscripten \
 && cp softfloat.wasm input.wasm target/wasm32-unknown-emscripten/debug \
 && cd target/wasm32-unknown-emscripten/debug \
 && cp softfloat.wasm _dev_urandom \
 && touch output.wasm

RUN cd wasm_check/target/wasm32-unknown-emscripten/debug \
 && source /emsdk/emsdk_env.sh \
 && node /emscripten-module-wrapper/prepare.js wasm_check.js --analyze --memory-size=25 --run --asmjs --debug --out=stuff --file output.wasm --file input.wasm --file softfloat.wasm --file _dev_urandom

