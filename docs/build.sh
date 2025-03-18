#!/usr/bin/sh
set -e
python -m pip install -r requirements.txt
cd ..
cargo test -p omf --lib -- --ignored update_schema_docs
cargo doc --no-deps
cd omf-python
cargo run --bin stub_gen
cargo build
python -m maturin develop
cd ..
python -m mkdocs build
mv ./target/doc ./site/rust
rm ./site/rust/.lock
