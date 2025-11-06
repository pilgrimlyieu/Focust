# GitHub Copilot Instructions for Focust

## Project Overview

Focust is a cross-platform break reminder application built with Tauri 2 (Rust backend) and Vue 3 (TypeScript frontend). It helps users take regular breaks with customizable schedules, beautiful themes, and intelligent reminders.

**Note**: This is an early development project and the author's first Rust project. Currently tested primarily on Windows, but designed for cross-platform support (Windows, macOS, Linux).

## Technology Stack

### Backend (Rust)
- **Tauri 2.x** - Desktop application framework
- **Tokio** - Async runtime
- **Serde + TOML** - Configuration serialization
- **ts-rs** - TypeScript type generation from Rust types
- **user-idle2** - System idle detection
- **tracing** - Structured logging
- **rodio** - Audio playback

### Frontend (Vue 3)
- **Vue 3.5** + **TypeScript 5.9** (strict mode)
- **Pinia** - State management
- **Vue I18n** - Internationalization (English and Chinese)
- **Tailwind CSS 4 + DaisyUI** - UI styling
- **Vite 7** - Build tool and dev server
- **Vitest** - Testing framework
- **Biome** - Linting and formatting (replaces ESLint/Prettier)

### Build Tools
- **Just** - Command runner (see `justfile` for available commands)
- **Cargo** - Rust package manager
- **npm/Bun** - JavaScript package manager (Bun preferred)

## Architecture

### Communication Pattern
- **Frontend → Backend**: Tauri commands via `invoke()`
- **Backend → Frontend**: Tauri events via `emit()` and `listen()`
- **Type Safety**: End-to-end type safety using ts-rs to generate TypeScript types from Rust structs
- **Persistence**: TOML configuration files stored in platform-specific config directory

### Project Structure

```
Focust/
├── src/                    # Frontend (Vue 3 + TypeScript)
│   ├── components/         # Vue components
│   │   ├── settings/       # Settings panel components
│   │   ├── ui/             # Reusable UI components
│   │   └── icons/          # Icon components
│   ├── composables/        # Vue composables
│   ├── stores/             # Pinia state management
│   ├── views/              # Main view components
│   ├── i18n/               # Internationalization files
│   ├── types/              # Type definitions
│   │   └── generated/      # Auto-generated from Rust (DO NOT EDIT)
│   └── utils/              # Utility functions
│
├── src-tauri/              # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── cmd/            # Tauri command handlers
│   │   ├── config/         # Configuration management
│   │   ├── core/           # Core business logic
│   │   │   ├── audio/      # Audio playback
│   │   │   ├── schedule.rs # Schedule types
│   │   │   ├── suggestions.rs # Suggestion system
│   │   │   └── theme.rs    # Theme types
│   │   ├── scheduler/      # Break scheduling engine
│   │   ├── platform/       # Platform integrations
│   │   │   ├── tray.rs     # System tray
│   │   │   ├── hotkey.rs   # Global hotkeys
│   │   │   └── notifications.rs # System notifications
│   │   └── utils/          # Utility functions
│   ├── assets/sounds/      # Built-in audio files
│   └── tests/              # Integration tests
│
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # Architecture documentation
│   ├── CONFIGURATION.md    # Configuration reference
│   └── QUICKSTART.md       # Quick start guide
│
└── justfile                # Command definitions
```

## Development Workflow

### Setup
```bash
# Install dependencies (use npm if bun not available)
bun install
# or: npm install

# Setup Rust dependencies
cd src-tauri && cargo check && cd ..

# Or use just
just setup
```

### Development Commands
```bash
# Start dev server
just dev
# or: bun run tauri dev

# Format code
just format      # Both frontend and backend
just format-front  # Frontend only
just format-back   # Backend only

# Lint/check code
just check       # All checks
just check-front # Frontend checks
just check-back  # Backend checks

# Run tests
just test-all       # All tests
just test-front-all # Frontend tests
just test-back-all  # Backend tests
```

## Coding Standards

### General Principles
1. **Write clean, readable code**: Prioritize clarity over cleverness
2. **Comment complex logic**: Explain the "why", not just the "what"
3. **Keep functions small**: Each function should do one thing well
4. **Use meaningful names**: Self-documenting variable, function, and type names
5. **Test your code**: Add tests for new features and bug fixes
6. **Type safety first**: Leverage TypeScript strict mode and Rust's type system

### Rust Code

**Style:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (via `just format-back`)
- Use `cargo clippy` for linting

**Naming Conventions:**
```rust
// Types: PascalCase
struct AppConfig { }
enum AudioSource { }

// Functions and variables: snake_case
fn load_config() -> Result<AppConfig> { }
let user_name = "Alice";

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
```

**Error Handling:**
- Use `Result<T, E>` for recoverable errors
- Use `?` operator for error propagation
- Avoid `unwrap()` in library code (acceptable in tests)

**TypeScript Type Generation:**
```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]  // JavaScript convention
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    pub check_for_updates: bool,
    pub theme_mode: String,
}
```

**Documentation:**
- Document public functions with doc comments (`///`)
- Include examples for complex APIs
- Describe parameters with `# Arguments` section
- Document return values and errors

### TypeScript/Vue Code

**Style:**
- Use Biome for formatting and linting (configured in `biome.jsonc`)
- Enable TypeScript strict mode
- Import paths use `@/` alias for `src/` directory

**Naming Conventions:**
```typescript
// Types and Interfaces: PascalCase
interface UserConfig {
  userName: string;
}
type AudioSource = "builtin" | "file";

// Functions and variables: camelCase
function loadConfig(): UserConfig { }
const userName = "Alice";

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES = 3;

// Components: PascalCase (Vue SFC files)
// Example: SettingsPanel.vue, BreakWindow.vue
```

**Vue 3 Composition API:**
- Use `<script setup lang="ts">` syntax
- Define props with TypeScript interfaces
- Use composables for reusable logic
- Leverage Pinia stores for shared state

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import type { AppConfig } from "@/types";

interface Props {
  config: AppConfig;
  readonly?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
});

// Component logic
</script>
```

**Imports:**
- Group imports: Vue imports, third-party libraries, local imports
- Use type imports when possible: `import type { ... }`
- Organize imports automatically with Biome

### Testing

**Frontend (Vitest):**
- Test files: `*.test.ts` or `*.spec.ts` alongside source files
- Use Vue Test Utils for component testing
- Mock Tauri commands when testing components that use them

**Backend (Rust):**
- Test files: `tests/` directory or inline `#[cfg(test)]` modules
- Use `tempfile` crate for temporary file operations in tests
- Test public APIs and critical logic paths

### Configuration

**Application Config:**
- Stored in TOML format in platform-specific config directory
- Defined in `src-tauri/src/config/models.rs`
- Supports partial config updates
- Always validate config after loading

**Type Generation:**
- Rust types with `#[derive(TS)]` auto-generate TypeScript types
- Generated types go to `src/types/generated/` (DO NOT EDIT MANUALLY)
- Run build to regenerate types after Rust struct changes

## Common Patterns

### Tauri Commands
```rust
// Backend (src-tauri/src/cmd/)
#[tauri::command]
pub async fn get_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    config::load(&app_handle)
        .await
        .map_err(|e| e.to_string())
}
```

```typescript
// Frontend
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "@/types/generated/AppConfig";

const config = await invoke<AppConfig>("get_config");
```

### Tauri Events
```rust
// Backend - emit event
app_handle.emit("break-started", payload)?;
```

```typescript
// Frontend - listen to event
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen<BreakPayload>("break-started", (event) => {
  console.log("Break started:", event.payload);
});
```

### Internationalization
```vue
<template>
  <div>{{ t('settings.title') }}</div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>
```

## Important Notes

### Platform Compatibility
- Primary testing on Windows
- macOS and Linux support is designed but less tested
- Consider platform-specific code paths in `src-tauri/src/platform/`
- Use conditional compilation: `#[cfg(windows)]`, `#[cfg(target_os = "linux")]`, etc.

### Generated Files
- **DO NOT EDIT**: Files in `src/types/generated/` are auto-generated from Rust
- Re-generate by running the build process
- Changes should be made in the Rust source structs

### File Paths
- Use Tauri's path APIs for file operations
- Respect platform-specific directory structures
- Use `tauri-plugin-fs` for filesystem access

### Breaking Changes
- This project is in early development
- Breaking changes may occur between versions
- Document breaking changes in commit messages
- Update RELEASE_NOTE.md for significant changes

## Documentation

- **ARCHITECTURE.md**: System architecture and design decisions
- **CONFIGURATION.md**: Configuration file format and options
- **CONTRIBUTING.md**: Contribution guidelines and workflow
- **QUICKSTART.md**: Quick start guide for users
- **README.md**: Project overview and setup instructions

## Helpful Resources

- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [TypeScript Documentation](https://www.typescriptlang.org/)
- [Biome Documentation](https://biomejs.dev/)
- [Just Documentation](https://just.systems/)

## Getting Help

- Check existing issues and documentation first
- Review `CONTRIBUTING.md` for guidelines
- Open an issue for bugs or feature requests
- Platform-specific contributions are especially welcome!
