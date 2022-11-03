#!/bin/bash
set -euo pipefail

WC='\033[1;95m'
NC='\033[0m'

echo -e "${WC}1. cargo build${NC}"
cargo build --release

echo -e "${WC}2. cargo clippy${NC}"
cargo clippy

echo -e "${WC}3. cargo fmt${NC}"
cargo fmt

echo -e "${WC}4. cargo test${NC}"
cargo test --lib
