#!/usr/bin/env bash

RM_WASM_PATH=target/wasm32-wasip1/release/rm.wasm
wasmtime --dir `pwd` --env PWD=`pwd` $RM_WASM_PATH "$@"