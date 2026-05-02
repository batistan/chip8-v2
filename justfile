build-wasm:
    wasm-pack build chip8-wasm --target bundler

dev: build-wasm
    cd www && npm run dev
