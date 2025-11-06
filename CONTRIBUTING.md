# Contributing to Focust

Thank you for your interest in contributing to Focust! Whether you're fixing bugs, adding features, improving documentation, or helping with translations, your contributions are warmly welcomed.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Areas Needing Help](#areas-needing-help)
- [Getting Help](#getting-help)

---

## Getting Started

### Prerequisites

Before you start contributing, make sure you have:

1. **Git** - [Download Git](https://git-scm.com/)
2. **Node.js** (v18+) or **Bun** (recommended) - [Download Bun](https://bun.sh/)
3. **Rust** (latest stable) - [Install Rustup](https://rustup.rs/)
4. **Just** (optional but recommended) - `cargo install just`
5. **Platform-specific dependencies** - See [README.md](README.md#building-from-source)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Focust.git
   cd Focust
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/pilgrimlyieu/Focust.git
   ```

### Initial Setup

```bash
# Install dependencies
bun install

# Setup Rust dependencies
cd src-tauri
cargo check
cd ..

# Or use Just
just setup
```

---

## Development Environment

### Recommended IDE

**Visual Studio Code** with the following extensions:

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust language support
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri development tools
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) - Vue 3 support
- [Biome](https://marketplace.visualstudio.com/items?itemName=biomejs.biome) - Formatting and linting

### Running the Development Server

```bash
# Start with hot-reload
bun run tauri dev

# Or use Just
just dev
```

The settings window should be opened in tray tab. To test break windows:
1. In the settings UI, adjust break intervals to short durations (e.g., 30 seconds)
2. Wait for the break to trigger, or use the "Trigger Break" test command in development

---

## Project Structure

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
│   ├── i18n/               # Internationalization
│   ├── types/              # Type definitions & utilities
│   │   └── generated/      # Auto-generated from Rust
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
│   └── CONFIGURATION.md    # Configuration reference
│
├── justfile                # Command definitions
└── README.md               # Project readme
```

For detailed architecture information, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Development Workflow

### Creating a New Branch

Always create a new branch for your work:

```bash
# Update your fork
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### Syncing with Upstream

Regularly sync your fork with the upstream repository:

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

### Running Tests

Before submitting changes, run all tests:

```bash
# Run all tests
just test-all

# Run frontend tests only
just test-front-all

# Run backend tests only
just test-back-all

# Run specific test
just test-back scheduler_test
```

### Code Quality Checks

Before committing, ensure your code meets quality standards:

```bash
# Format code
just format

# Run linters
just lint

# Type checking
just check

# Or run all at once
just pre-commit
```

---

## Coding Standards

### General Principles

1. **Write clean, readable code**: Prioritize clarity over cleverness
2. **Comment complex logic**: Explain the "why", not just the "what"
3. **Keep functions small**: Each function should do one thing well
4. **Use meaningful names**: Variables, functions, and types should be self-documenting
5. **Test your code**: Add tests for new features and bug fixes

### Language Standards

#### Rust Code

**Style:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
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

// Private fields: prefix with underscore if unused
struct Internal {
    _unused_field: String,
}
```

**Error Handling:**
```rust
// Use Result<T, E> for recoverable errors
fn parse_config(path: &str) -> Result<AppConfig, ConfigError> {
    // ...
}

// Use ? operator for error propagation
let config = load_file(path)?;

// Avoid unwrap() in library code
// In tests and examples, unwrap() is acceptable
#[cfg(test)]
fn test_parsing() {
    let config = parse_config("test.toml").unwrap();
}
```

**Async Code:**
```rust
// Use async/await with tokio
async fn fetch_data() -> Result<Data> {
    let response = reqwest::get("https://api.example.com")
        .await?;
    let data = response.json().await?;
    Ok(data)
}

// Spawn long-running tasks
tauri::async_runtime::spawn(async move {
    // Long-running work
});
```

**Type Exports for TypeScript:**
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
```rust
/// Loads the application configuration from disk.
///
/// This function attempts to load the config file from the platform-specific
/// configuration directory. If the file doesn't exist, it creates a default
/// configuration.
///
/// # Arguments
///
/// * `app_handle` - Handle to the Tauri application
///
/// # Returns
///
/// Returns the loaded or default configuration
///
/// # Examples
///
/// ```
/// let config = load_config(&app_handle).await;
/// ```
pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    // Implementation
}
```

#### TypeScript/Vue Code

**Style:**
- Use Biome for formatting and linting (configured in `biome.json`)
- Use TypeScript strict mode

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
// SettingsPanel.vue, BreakWindow.vue
```

**Vue 3 Composition API:**
```vue
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import type { AppConfig } from "@/types";

// Props with TypeScript
interface Props {
  config: AppConfig;
  readonly?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
});

// Reactive state
const count = ref(0);
const doubled = computed(() => count.value * 2);

// Lifecycle hooks
onMounted(() => {
  console.log("Component mounted");
});

// Expose for testing
defineExpose({
  count,
  doubled,
});
</script>

<template>
  <div>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubled }}</p>
  </div>
</template>
```

**State Management (Pinia):**
```typescript
import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useConfigStore = defineStore("config", () => {
  // State
  const config = ref<AppConfig | null>(null);
  const loading = ref(false);

  // Getters
  const isDarkMode = computed(() => {
    return config.value?.themeMode === "dark";
  });

  // Actions
  async function loadConfig() {
    loading.value = true;
    try {
      config.value = await invoke<AppConfig>("get_config");
    } finally {
      loading.value = false;
    }
  }

  return {
    config,
    loading,
    isDarkMode,
    loadConfig,
  };
});
```

**Error Handling:**
```typescript
// Handle errors gracefully
try {
  await invoke("save_config", { config });
  showToast("success", "Settings saved");
} catch (error) {
  console.error("Failed to save config:", error);
  showToast("error", "Failed to save settings");
}

// Type guards for generated types
import { isBreakPayload } from "@/types";

if (isBreakPayload(data)) {
  // TypeScript knows data is BreakPayload
  startBreak(data);
}
```

**Documentation:**
```typescript
/**
 * Loads the user configuration from the backend.
 *
 * @returns {Promise<UserConfig>} A promise that resolves to the user configuration.
 * @throws An error if the configuration cannot be loaded.
 * @example
 * ```typescript
 * const config = await loadUserConfig();
 * console.log(config.themeMode);
 * ```
 */
export async function loadUserConfig(): Promise<UserConfig> {
  const config = await invoke<UserConfig>("get_user_config");
  return config;
}
```

### Testing Standards

#### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let result = add(2, 3);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_error_handling() {
        let result = divide(10, 0);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_function() {
        let data = fetch_data().await.unwrap();
        assert!(!data.is_empty());
    }
}
```

#### TypeScript/Vue Tests

```typescript
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import MyComponent from "@/components/MyComponent.vue";

describe("MyComponent", () => {
  it("renders properly", () => {
    const wrapper = mount(MyComponent, {
      props: { title: "Test" },
    });
    expect(wrapper.text()).toContain("Test");
  });

  it("handles click events", async () => {
    const wrapper = mount(MyComponent);
    await wrapper.find("button").trigger("click");
    expect(wrapper.emitted("submit")).toBeTruthy();
  });
});
```

---

## Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no logic changes)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions or modifications
- `chore`: Build process, dependencies, tooling

### Scope

Optional, indicates the area of change:
- `config`: Configuration system
- `scheduler`: Break scheduler
- `ui`: User interface
- `audio`: Audio system
- `tray`: System tray
- `i18n`: Internationalization

### Examples

```
feat(scheduler): add postpone break functionality

Implement the ability to postpone breaks by a configurable duration.
Users can now use the postpone button or global hotkey to delay
the next break.

Closes #42

---

fix(tray): fix tray menu not updating on pause

The tray menu was not reflecting the paused state correctly.
Fixed by adding proper state synchronization.

Fixes #38

---

docs(architecture): update backend architecture documentation

Added details about the event-driven scheduler and improved
the module organization diagrams.

---

chore(deps): update tauri to 2.9.2

Updated Tauri and related plugins to the latest versions.
```

---

## Pull Request Process

### Before Submitting

1. **Update your branch** with the latest upstream changes
2. **Run all tests** and ensure they pass
3. **Format and lint** your code
4. **Update documentation** if needed
5. **Add tests** for new features or bug fixes

### Submitting a PR

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature
   ```

2. **Open a Pull Request** on GitHub:
   - Use a clear, descriptive title
   - Reference related issues (e.g., "Closes #123", "Fixes #456")
   - Describe your changes in detail
   - Add screenshots for UI changes
   - List any breaking changes

3. **Fill out the PR template**:
   - The PR template will automatically appear when you create a pull request
   - Complete all relevant sections (description, type of change, testing, checklist)
   - Check all applicable boxes in the checklist before requesting review
   - See `.github/pull_request_template.md` for the full template

### Review Process

1. **Automated Checks**: CI/CD will run tests and linting
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, your PR will be merged

### After Merging

1. **Delete your branch** (optional):
   ```bash
   git branch -d feature/your-feature
   git push origin --delete feature/your-feature
   ```

2. **Update your local main**:
   ```bash
   git checkout main
   git pull upstream main
   ```

---

## Areas Needing Help

### High Priority

- **Platform Testing**: Test on macOS and Linux, report platform-specific issues
- **Bug Fixes**: Check [open issues](https://github.com/pilgrimlyieu/Focust/issues?q=sort%3Aupdated-desc+is%3Aissue+is%3Aopen+%28label%3ABUG+OR+type%3ABug%29) labeled `bug`
- **Documentation**: Improve existing docs, add missing documentation

### Medium Priority

- **Translations**: Add support for new languages
- **UI/UX Improvements**: Design enhancements, accessibility improvements
- **Performance**: Profile and optimize slow operations
- **Test Coverage**: Add tests for untested code paths

### Feature Requests

Check [open issues](https://github.com/pilgrimlyieu/Focust/issues) labeled `enhancement` or `feature-request`. Feel free to propose new features via discussions.

---

## Getting Help

### Resources

- **Documentation**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/CONFIGURATION.md](docs/CONFIGURATION.md)
- **API Reference**: Generated TypeScript types in `src/types/generated/`
- **Examples**: Check existing code for similar implementations

### Communication

- **GitHub Issues**: Report bugs or request features
- **GitHub Discussions**: Ask questions, share ideas
- **Code Comments**: Check inline comments for context
- **PR Comments**: Ask questions during code review

### Tips for New Contributors

1. **Start small**: Fix typos, improve documentation, add tests
2. **Read existing code**: Understand the patterns and conventions
3. **Ask questions**: Don't hesitate to ask for help
4. **Be patient**: Code review takes time
5. **Learn from feedback**: Use reviews as learning opportunities

---

## Development Tips

### Quick Commands (Just)

```bash
# Pre-commit checks (format, lint, check)
just pre-commit

# Watch mode for tests
bun run test:ui         # Frontend tests with UI
cargo watch -x test     # Backend tests (requires cargo-watch)

# Clean and rebuild
just clean
just build
```

### Debugging

**Rust Backend:**
```rust
// Add debug logs
tracing::debug!("Variable value: {value:?}");
tracing::info!("Processing started");
tracing::warn!("Potential issue detected");
tracing::error!("Operation failed: {error}");
```

**Frontend:**
```typescript
// Console debugging
console.log("[Component] State:", state);
console.warn("[Store] Invalid data:", data);

// Vue Devtools
// Install Vue Devtools browser extension for reactive debugging
```

### Common Issues

**Issue: Types out of sync**
```bash
# Regenerate TypeScript types from Rust
cd src-tauri
cargo test export_bindings
```

**Issue: Build fails after pulling changes**
```bash
# Clean and reinstall dependencies
just clean
bun install
cargo clean
just setup
```

**Issue: Hot reload not working**
```bash
# Restart dev server
# Kill any running tauri dev processes
just dev
```

---

## License

By contributing to Focust, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

**Thank you for contributing to Focust! Your efforts help make break reminders better for everyone.** 🎉
