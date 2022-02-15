#!/usr/bin/env bash
set -e

script_path="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )/"
echo script_path: ${script_path}

output_path=${script_path}

cargo +nightly build \
    --lib \
    --release \
    -Z unstable-options \
    --out-dir=${output_path}

mkdir -p ${output_path:?}/data/
rm -rf ${output_path:?}/data/*

( cd ${script_path} && tarantool app.lua )
