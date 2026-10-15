#!/usr/bin/env bash
# Build command for Cloudflare Workers Builds. Cloudflare's build machine has no Rust, so install it first
# (a minimal toolchain, cached between builds when possible), then export the site into public/.
# Needs the SITE_URL build variable, for example https://guideboy.fyi
set -euo pipefail
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi
cargo run --release --locked --bin export -- public
