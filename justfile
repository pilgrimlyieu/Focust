set dotenv-load
set shell := ["bash", "-c"]
set windows-shell := ["powershell", "-NoLogo", "-Command"]

RUST_DIR := "src-tauri"

TAURI_CMD := "bun run tauri"
RM_CMD := if os_family() == "windows" { "Remove-Item -Force -Recurse -ErrorAction SilentlyContinue" } else { "rm -rf" }

alias s := setup
alias d := dev

alias b := build
alias bd := build-debug

alias cl := clean
alias clf := clean-front
alias clb := clean-back

alias l := lint
alias lf := lint-front
alias lb := lint-back

alias f := format
alias ff := format-front
alias fb := format-back

alias c := check
alias cf := check-front
alias cb := check-back

alias fi := fix
alias fif := fix-front
alias fib := fix-back

alias ta := test-all
alias tl := test-lib
alias tfa := test-front-all
alias tf := test-front
alias tba := test-back-all
alias tb := test-back

alias u := update
alias uf := update-front
alias ub := update-back

alias adf := add-dep-front
alias adb := add-dep-back

# -----------------------------------------------------------------------------
# Core Development & Build Commands
# -----------------------------------------------------------------------------

# List available commands
@_default:
    just --list --unsorted
    echo ""
    echo "💡 Use 'just setup' to prepare your environment."
    echo "💡 Use 'just dev' to start the development server."

# Setup the project environment
@setup:
    echo "🚀 Setting up project dependencies..."
    -bun install
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml
    echo "✅ Setup complete! You can now run 'just dev'."

# Start the development server
@dev:
    echo "▶️ Starting Tauri development server..."
    {{ TAURI_CMD }} dev

# Build the Tauri application
[group: "build"]
@build:
    echo "📦 Building Tauri application (release mode)..."
    {{ TAURI_CMD }} build

# Build the Tauri application in debug mode
[group: "build"]
@build-debug:
    echo "📦 Building Tauri application (debug mode)..."
    {{ TAURI_CMD }} build --debug

# Clean project artifacts
[group: "clean"]
[confirm: "Are you sure you want to clean the project artifacts? This will remove all build outputs."]
@clean:
    echo "🧹 Cleaning project artifacts..."
    -{{ RM_CMD }} dist
    cd {{ RUST_DIR }}; cargo clean
    echo "✅ Clean complete!"

# Clean front-end artifacts
[group: "clean"]
[confirm: "Are you sure you want to clean the front-end artifacts? This will remove all build outputs."]
@clean-front:
    echo "🧹 Cleaning front-end artifacts..."
    -{{ RM_CMD }} dist
    echo "✅ Front-end clean complete!"

# Clean back-end artifacts
[group: "clean"]
[confirm: "Are you sure you want to clean the back-end artifacts? This will remove all build outputs."]
[working-directory: 'src-tauri']
@clean-back:
    echo "🧹 Cleaning back-end artifacts..."
    cargo clean
    echo "✅ Back-end clean complete!"


# -----------------------------------------------------------------------------
# Formatting, Checking, Linting, and Fixing
# -----------------------------------------------------------------------------

# Formatting
[group: "format"]
@format:
    echo "💅 Formatting code..."
    -bunx biome format --write .
    cargo fmt --manifest-path {{ RUST_DIR }}/Cargo.toml --all
    echo "✅ Formatting complete!"

# Front-end specific formatting
[group: "format"]
@format-front:
    echo "💅 Formatting front-end code..."
    bunx biome format --write .
    echo "✅ Front-end formatting complete!"

# Back-end specific formatting
[group: "format"]
@format-back:
    echo "💅 Formatting back-end code..."
    cargo fmt --manifest-path {{ RUST_DIR }}/Cargo.toml --all
    echo "✅ Back-end formatting complete!"

# Checking
[group: "check"]
@check:
    echo "🧐 Running static analysis..."
    -bunx biome check .
    -bunx tsc --noEmit
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Checks complete!"

# Front-end specific checks
[group: "check"]
@check-front:
    echo "🧐 Running front-end checks..."
    -bunx biome check .
    bunx tsc --noEmit
    echo "✅ Front-end checks complete!"

# Back-end specific checks
[group: "check"]
@check-back:
    echo "🧐 Running back-end checks..."
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Back-end checks complete!"

# Linting
[group: "lint"]
@lint:
    echo "🔍 Running linters..."
    -bunx biome lint .
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Linting complete!"

# Front-end specific linting
[group: "lint"]
@lint-front:
    echo "🔍 Running front-end linters..."
    bunx biome lint .
    echo "✅ Front-end linting complete!"

# Back-end specific linting
[group: "lint"]
@lint-back:
    echo "🔍 Running back-end linters..."
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Back-end linting complete!"

# Fixing
[group: "fix"]
@fix:
    echo "🛠️ Fixing code issues..."
    bunx biome check --write .
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets --fix --allow-dirty
    echo "✅ Fixing complete!"

# Front-end specific fixing
[group: "fix"]
@fix-front:
    echo "🛠️ Fixing front-end code issues..."
    bunx biome check --write .
    echo "✅ Front-end fixing complete!"

# Back-end specific fixing
[group: "fix"]
@fix-back:
    echo "🛠️ Fixing back-end code issues..."
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets --fix --allow-dirty
    echo "✅ Back-end fixing complete!"

# -----------------------------------------------------------------------------
# Testing
# -----------------------------------------------------------------------------

# Run all tests
# Note: Back-end tests use --test-threads=1 due to tokio::time::pause()
[group: "test"]
@test-all:
    echo "🧪 Running tests..."
    -bun run test:run
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace -- --test-threads=1
    echo "✅ Tests complete!"

# Run library tests only
[group: "test"]
@test-lib *tests:
    echo "🧪 Running library tests..."
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --lib {{ tests }}
    echo "✅ Library tests complete!"

# Run all front-end tests
[group: "test"]
@test-front-all:
    echo "🧪 Running front-end tests..."
    bun run test:run
    echo "✅ Front-end tests complete!"
    
# Run front-end tests
[group: "test"]
@test-front +tests:
    echo "🧪 Running front-end tests..."
    bun run test:run {{ tests }}
    echo "✅ Front-end tests complete!"

# Run all back-end tests
# Note: Uses --test-threads=1 because scheduler tests use tokio::time::pause()
# which affects global time and cannot run concurrently
[group: "test"]
@test-back-all:
    echo "🧪 Running back-end tests..."
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace -- --test-threads=1
    echo "✅ Back-end tests complete!"

# Run back-end tests
[group: "test"]
@test-back +tests:
    echo "🧪 Running back-end tests..."
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace {{ tests }}
    echo "✅ Back-end tests complete!"

# -----------------------------------------------------------------------------
# Helper Recipes
# -----------------------------------------------------------------------------

# Update dependencies
[group: "update-dependencies"]
@update:
    echo "⬆️ Updating dependencies..."
    -bun update
    cargo update --manifest-path {{ RUST_DIR }}/Cargo.toml
    echo "✅ Dependencies updated!"

# Update front-end dependencies
[group: "update-dependencies"]
@update-front:
    echo "⬆️ Updating front-end dependencies..."
    bun update
    echo "✅ Front-end dependencies updated!"

# Update back-end dependencies
[group: "update-dependencies"]
@update-back:
    echo "⬆️ Updating back-end dependencies..."
    cargo update --manifest-path {{ RUST_DIR }}/Cargo.toml
    echo "✅ Back-end dependencies updated!"

# Add front-end dependency
[group: "add-dependency"]
@add-dep-front +deps:
    echo "⬆️ Adding front-end dependencies..."
    bun add {{ deps }}
    echo "✅ Front-end dependencies added!"

# Add back-end dependency
[group: "add-dependency"]
@add-dep-back +deps:
    echo "⬆️ Adding back-end dependencies..."
    cargo add {{ deps }} --manifest-path {{ RUST_DIR }}/Cargo.toml
    echo "✅ Back-end dependencies added!"

# -----------------------------------------------------------------------------
# Git
# -----------------------------------------------------------------------------

# Check before committing
[group: "git"]
@pre-commit-checks:
    echo "🔒 Running frontend checks..."
    bunx biome check .
    bunx tsc --noEmit
    echo "✅ Frontend checks passed!"
    echo "🔒 Running backend checks..."
    cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-target
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets -- -D warnings
    echo "✅ Backend checks passed!"

# Check before committing
[group: "git"]
@pre-commit-checks-all:
    echo "🔒 Running front-end checks..."
    -bunx biome check .
    -bunx tsc --noEmit
    echo "✅ Front-end checks passed!"
    echo "🔒 Running back-end checks..."
    -cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
    -cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    -cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets -- -D warnings
    echo "✅ Back-end checks passed!"

# Fix before committing
[group: "git"]
@pre-commit-fixes:
    echo "💅 Formatting front-end code..."
    -bunx biome format --write .
    echo "✅ Front-end formatting applied!"
    echo "💅 Formatting back-end code..."
    -cargo fmt --manifest-path {{ RUST_DIR }}/Cargo.toml --all
    echo "✅ Back-end formatting applied!"
    echo "🛠️ Fixing front-end code issues..."
    -bunx biome check --write .
    echo "✅ Front-end fixing complete!"
    echo "🛠️ Fixing back-end code issues..."
    -cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets --fix --allow-dirty
    echo "✅ Back-end fixing complete!"