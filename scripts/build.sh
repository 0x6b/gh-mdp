#!/bin/bash

set -euo pipefail

mkdir -p ./dist

TARGET_TRIPLE=${TARGET_TRIPLE:-x86_64-unknown-linux-gnu}
OS_ARCH=${OS_ARCH:-linux-amd64}

cargo build --release --locked --target "${TARGET_TRIPLE}"

# Handle Windows .exe extension
if [[ "${TARGET_TRIPLE}" == *"windows"* ]]; then
    mv "target/${TARGET_TRIPLE}/release/gh-mdp.exe" "./dist/${OS_ARCH}"
else
    mv "target/${TARGET_TRIPLE}/release/gh-mdp" "./dist/${OS_ARCH}"
fi
