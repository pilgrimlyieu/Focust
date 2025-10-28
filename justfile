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

alias ta := test-all
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
# Linting, Formatting, Checking
# -----------------------------------------------------------------------------

# Linting
[group: "lint"]
@lint:
    echo "🔍 Running linters..."
    -bunx biome check .
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Linting complete!"

# Front-end specific linting
[group: "lint"]
@lint-front:
    echo "🔍 Running front-end linters..."
    bunx biome check .
    echo "✅ Front-end linting complete!"

# Back-end specific linting
[group: "lint"]
@lint-back:
    echo "🔍 Running back-end linters..."
    cargo clippy --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Back-end linting complete!"

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
    echo "🧐 Running type checks and static analysis..."
    -bunx biome check .
    -bunx tsc --noEmit
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Checks complete!"

# Front-end specific checks
[group: "check"]
@check-front:
    echo "🧐 Running front-end type checks..."
    bunx tsc --noEmit
    echo "✅ Front-end type checks complete!"

# Back-end specific checks
[group: "check"]
@check-back:
    echo "🧐 Running back-end type checks..."
    cargo check --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace --all-targets
    echo "✅ Back-end type checks complete!"

# -----------------------------------------------------------------------------
# Testing
# -----------------------------------------------------------------------------

# Run all tests
[group: "test"]
@test-all:
    echo "🧪 Running tests..."
    -bun run test:run
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace
    echo "✅ Tests complete!"

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
[group: "test"]
@test-back-all:
    echo "🧪 Running back-end tests..."
    cargo test --manifest-path {{ RUST_DIR }}/Cargo.toml --workspace
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
@pre-commit:
    echo "🔒 Running pre-commit checks..."
    just format
    just check
    echo "✅ Pre-commit checks passed!"