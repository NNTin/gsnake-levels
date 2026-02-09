#!/usr/bin/env bash
set -euo pipefail

mkdir -p target/llvm-cov

cargo llvm-cov \
  --workspace \
  --all-targets \
  --fail-under-lines 80 \
  --lcov \
  --output-path target/llvm-cov/lcov.info
