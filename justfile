# Shows all available recipes
default:
  @just --list

# Build the HTML, JS, and WASM for the overlay
wasm-build:
  cd ./crates/overlay && rm -rf ./dist; trunk build --filehash false

# Build the HTML, JS, and WASM for the overlay and then hot reload on save
wasm-serve:
  cd ./crates/overlay && trunk serve --filehash false

# Rebuild HTML, JS and WASM, then restart GUI
run:
  just wasm-build
  cargo run --bin schnose-gsi-client

# vim: et ts=2 sw=2 sts=2 ai si
