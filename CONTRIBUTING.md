# Contributing

## One-time setup

### Toolchain

- **Rust** ≥ 1.85 (driven by `rust-toolchain.toml`; `rustup` will pick this up automatically)
- **Node** 22 (matches CI; use `nvm use 22` if you have nvm)
- **wasm-pack** — install via `cargo install wasm-pack` or `cargo binstall wasm-pack`
- **just** (optional) — for the recipes in `justfile`

### Build the wasm package

`www/` consumes the wasm artifact at `chip8-wasm/pkg/`, which is gitignored.
Build it once:

```sh
just build-wasm
# or:
wasm-pack build chip8-wasm --target bundler
```

Re-run any time you change `chip8-core/` or `chip8-wasm/`.

### Enable the pre-commit hook

The repo ships a path-aware pre-commit hook at `.githooks/pre-commit` that runs
the same lint/typecheck/build gates as CI, scoped to whichever stack your staged
files touch. Tests are skipped (CI covers them).

Enable once per clone:

```sh
git config core.hooksPath .githooks
```

This is opt-in by design — git won't run hooks from a tracked directory until
you point `core.hooksPath` at it.

To bypass the hook for a specific commit (use sparingly):

```sh
git commit --no-verify
```

## Running checks manually

```sh
# Rust
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --exclude chip8-wasm -- -D warnings
cargo clippy -p chip8-wasm --target wasm32-unknown-unknown -- -D warnings
cargo build --workspace --exclude chip8-wasm
cargo test --workspace --exclude chip8-wasm

# www
cd www
npm run lint
npm run typecheck
npm run build
npm test -- --run
```

## CI

CI runs on every pull request (and on every push to a branch with an open PR).
Branches without an open PR don't trigger CI. The workflow at
`.github/workflows/ci.yml` has two jobs gated by `dorny/paths-filter`:

- **rust** — runs when files under `chip8-core/`, `chip8-wasm/`, or the cargo
  config change.
- **www** — runs when files under `www/`, `chip8-core/`, or `chip8-wasm/`
  change (the wasm crates are inputs to the www build).
