@echo off
python -m pip install -r requirements.txt || exit /b
cd ..
cargo test -p omf --lib -- --ignored update_schema_docs || exit /b
cargo doc --no-deps || exit /b
python -m mkdocs build || exit /b
move target\doc site\rust || exit /b
del site\rust\.lock || exit /b
