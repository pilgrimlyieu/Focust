# GitHub Copilot Instructions for Focust

> **Purpose**: Self-contained guide for GitHub Copilot
> **Audience**: Developers and AI assistants working on Focust

## 📋 Project Overview

Focust is a cross-platform break reminder app built with **Tauri 2.9** (Rust backend) + **Vue 3.5** (TypeScript frontend). It helps users take regular breaks with customizable schedules and themes.

**Tech Stack**:
- **Backend**: Tauri 2.9, Rust, Tokio, Serde, ts-rs, rodio
- **Frontend**: Vue 3.5, TypeScript 5.9, Pinia, Tailwind CSS 4
- **Tools**: Just, Biome, Vitest, Cargo test

## 🏗️ Architecture (Self-Contained)

### System Overview

```
Frontend (Vue 3)  ←[invoke/emit]→  Tauri IPC  ←→  Rust Backend
     ↓                                            ↓
Pinia Stores                              SchedulerManager
     ↓                                            ↓
Settings/Break Views                  BreakScheduler + AttentionTimer
```

### Key Backend Modules (`src-tauri/src/`)

- **`cmd/`** - Tauri command handlers (23 commands)
- **`scheduler/`** - Event-driven scheduling (BreakScheduler + AttentionTimer)
- **`monitors/`** - Environment monitoring (Idle/DND/AppWhitelist)
- **`config/`** - TOML config with partial loading + auto migration
- **`platform/`** - System integration (tray, hotkeys, notifications)
- **`core/`** - Business logic (audio, schedule, themes)

### Key Frontend Modules (`src/`)

- **`stores/`** - Pinia state (configStore, schedulerStore)
- **`views/`** - Main views (SettingsApp, PromptApp)
- **`components/`** - Reusable UI components (lazy-loaded panels)
- **`types/`** - Type system (generated/ from ts-rs, guards, factories)

## 📋 Common Commands

```bash
just dev          # Start dev server
just format       # Format code (frontend + backend)
just check        # Type check + Clippy
just test-all     # Run all tests
just build        # Build production version
```

## 🎯 Coding Standards (Critical Rules)

### Rust

| Rule | Correct ✅ | Wrong ❌ | Why |
|------|-----------|----------|-----|
| **Format strings** | `tracing::info!("User {user_name}");` | `format!("User: {}", user_name)` | Direct interpolation is cleaner |
| **Error handling** | `config.load()?;` | `config.load().unwrap();` | Avoid panics, propagate errors |
| **Async tasks** | `tauri::async_runtime::spawn(...)` | `tokio::spawn(...)` | Avoid "Cannot start runtime" error |
| **Naming** | `fn calculate_next_event()` | `fn CalcNextEvent()` | Follow snake_case convention |
| **Imports** | `use std::fs;` then `fs::read()` | `use std::fs::read;` then `read()` | Keep namespace clarity |

**Error Handling Pattern**:
```rust
// Library code: Use Result<T, E>
pub fn load_config() -> Result<AppConfig, ConfigError> { }

// Application code: Use anyhow
fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    Ok(())
}
```

### TypeScript/Vue

| Rule | Correct ✅ | Wrong ❌ | Why |
|------|-----------|----------|-----|
| **Props** | `interface Props { config: AppConfig; }` | `defineProps(['config'])` | Type safety |
| **Type imports** | `import type { AppConfig } from '@/types';` | `import { AppConfig }...` | Avoid runtime imports |
| **Deep clone** | `safeClone(config)` | `JSON.parse(JSON.stringify(config))` | Handles BigInt correctly |
| **Naming** | `function loadConfig()` | `function LoadConfig()` | Follow camelCase |
| **Component setup** | `<script setup lang="ts">` | `<script lang="ts">` + `defineComponent` | Composition API preferred |

**Vue Component Pattern**:
```vue
<script setup lang="ts">
import type { AppConfig } from '@/types';

interface Props {
  config: AppConfig;
  readonly?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
});
</script>
```

## 🚨 Common Pitfalls & Solutions

| Issue | Cause | Solution | Location |
|-------|-------|----------|----------|
| **BigInt clone fails** | `JSON.parse/stringify` doesn't support BigInt | Use `safeClone()` from `@/utils/safeClone` | `src/utils/safeClone.ts` |
| **Event permission error** | Window not declared in capabilities | Add window label to `capabilities/default.json` | `src-tauri/capabilities/default.json` |
| **Tokio Runtime error** | Using `tokio::spawn` in Tauri | Use `tauri::async_runtime::spawn` instead | All async code |
| **`unwrap()` panic** | Calling `unwrap()` on `Option/Result` | Use `?` operator or `ok_or_else()` | All Rust code |
| **Type generation fails** | Rust struct changed but TS not updated | Run `cargo test export_bindings` | After changing `#[derive(TS)]` structs |

## 🧩 Code Templates

### Add Tauri Command

**Backend** (`src-tauri/src/cmd/xxx.rs`):
```rust
#[tauri::command]
pub async fn new_command(
    app_handle: tauri::AppHandle,
    param: String,
) -> Result<ReturnType, String> {
    // Implementation
    Ok(result)
}
```

**Frontend** (`src/xxx.ts`):
```typescript
import { invoke } from '@tauri-apps/api/core';
import type { ReturnType } from '@/types';

const result = await invoke<ReturnType>('new_command', { 
    param: 'value' 
});
```

**Register** (`src-tauri/src/cmd.rs`):
```rust
pub fn register_commands() -> /* ... */ {
    vec![
        // ...existing commands...
        new_command,
    ]
}
```

### Add Config Field

**Backend** (`src-tauri/src/config/models.rs`):
```rust
#[derive(Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    pub new_field: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // ...existing fields...
            new_field: true, // Set default value
        }
    }
}
```

**Regenerate types**:
```bash
cargo test export_bindings
```

**Frontend usage**:
```typescript
const config = useConfigStore();
config.draft.newField = false; // Auto-completed!
```

### Add Event Listener

**Backend emits**:
```rust
app_handle.emit("new-event", payload)?;
```

**Frontend listens**:
```typescript
import { listen } from '@tauri-apps/api/event';
import type { NewEventPayload } from '@/types';

const unlisten = await listen<NewEventPayload>('new-event', (event) => {
    console.log(event.payload);
});

// Cleanup on unmount
onUnmounted(() => unlisten());
```

**Add permission** (`src-tauri/capabilities/default.json`):
```json
{
  "windows": ["main", "settings"],
  "permissions": [
    {
      "identifier": "allow-custom-events",
      "allow": [
        { "event": "new-event" }
      ]
    }
  ]
}
```

## 🔄 IPC Communication Patterns

### Commands (Frontend → Backend)

**23 commands organized by module**:

| Module | Commands | Purpose |
|--------|----------|---------|
| **config** | `get_config`, `save_config`, `pick_background_image` | Config CRUD |
| **scheduler** | `pause_scheduler`, `resume_scheduler`, `postpone_break`, `skip_break`, `trigger_event`, `request_scheduler_status`, `reset_scheduler` | Scheduler control |
| **audio** | `play_builtin_audio`, `play_audio_file`, `stop_audio` | Audio control |
| **suggestions** | `get_suggestions`, `save_suggestions`, `get_suggestions_for_language` | Suggestion CRUD |
| **system** | `open_config_dir`, `open_log_dir`, `open_folder` | File operations |
| **window** | `open_settings_window`, `close_break_windows` | Window management |
| **autostart** | `get_autostart`, `set_autostart` | Autostart management |

### Events (Backend → Frontend)

| Event | Payload | When Emitted |
|-------|---------|--------------|
| `scheduler-status` | `SchedulerStatus` | Status changes (pause/resume, next event update) |
| `scheduler-event` | `PromptPayload` | Break or attention trigger |
| `scheduler-paused` | `()` | Scheduler paused |
| `scheduler-resumed` | `()` | Scheduler resumed |
| `postpone-limit-reached` | `()` | Max postpones reached |

## 🎨 Key Design Patterns

### 1. Type-Safe IPC with ts-rs

**Backend defines, frontend auto-generates**:
```rust
// Rust side
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    pub autostart: bool,
}
```

**Result** (auto-generated in `src/types/generated/AppConfig.ts`):
```typescript
export interface AppConfig {
  autostart: boolean;
}
```

**Never edit** `src/types/generated/` manually!

### 2. Draft/Original Config Pattern

**Problem**: User cancels changes  
**Solution**: Edit draft, save to original

```typescript
// Load
const original = await invoke('get_config');
const draft = safeClone(original);

// User edits draft
draft.autostart = true;

// Save
await invoke('save_config', { config: draft });
original = safeClone(draft); // Sync

// Cancel
draft = safeClone(original); // Revert
```

### 3. Partial Config Loading

**Problem**: New version adds field → old config breaks  
**Solution**: Parse as generic TOML, merge valid fields

```rust
// Load TOML as generic Value
let toml_value: toml::Value = toml::from_str(&content)?;

// Try full deserialize
match toml::from_str::<AppConfig>(&content) {
    Ok(config) => Ok(config),
    Err(_) => {
        // Partial merge with defaults
        let default = AppConfig::default();
        let merged = merge_partial(&toml_value, &default)?;
        Ok(merged)
    }
}
```

### 4. Bitflags for Pause Reasons

**Problem**: Multiple pause reasons (idle + DND + manual)  
**Solution**: Bitflags, resume only when all cleared

```rust
bitflags! {
    pub struct PauseReasons: u8 {
        const USER_IDLE     = 1 << 0;
        const DND           = 1 << 1;
        const MANUAL        = 1 << 2;
        const APP_EXCLUSION = 1 << 3;
    }
}
```

**Logic**:
- `add_pause_reason()` → Pause if first reason
- `remove_pause_reason()` → Resume if last reason removed

## 🧪 Testing

**Coverage**: 273 tests, 100% pass rate

**Run tests**:
```bash
just test-all              # All tests
cargo test --lib           # Rust unit tests
cargo test scheduler::     # Specific module
bun test                   # Frontend tests
```

**Test pattern** (Rust):
```rust
#[tokio::test(start_paused = true)]  // Simulate time
async fn break_timing_works() {
    // Arrange
    let scheduler = create_test_scheduler();
    
    // Act
    tokio::time::advance(Duration::from_secs(1200)).await;
    
    // Assert
    assert!(scheduler.should_break());
}
```

## Application Directories

Focust separates development and production environments using different directories:

**Production (Release) Builds**:
- **Windows**: `%APPDATA%\com.fesmoph.focust\` (config), `%LOCALAPPDATA%\com.fesmoph.focust\logs\` (logs)
- **macOS**: `~/Library/Application Support/com.fesmoph.focust/` (config), `~/Library/Logs/com.fesmoph.focust/` (logs)
- **Linux**: `~/.config/com.fesmoph.focust/` (config), `~/.local/share/com.fesmoph.focust/logs/` (logs)

**Development (Debug) Builds**:
- **Windows**: `%APPDATA%\com.fesmoph.focust.dev\` (config), `%LOCALAPPDATA%\com.fesmoph.focust\logs.dev\` (logs)
- **macOS**: `~/Library/Application Support/com.fesmoph.focust.dev/` (config), `~/Library/Logs/com.fesmoph.focust.dev/` (logs)
- **Linux**: `~/.config/com.fesmoph.focust.dev/` (config), `~/.local/share/com.fesmoph.focust/logs.dev/` (logs)

> **Note**: In debug builds the `.dev` suffix is appended to the final path component. This means that on Windows and Linux the logs directory becomes `.../com.fesmoph.focust/logs.dev`, while on macOS the logs directory becomes `~/Library/Logs/com.fesmoph.focust.dev/`. This prevents development work from affecting production configurations and logs. Implementation: `src-tauri/src/utils/paths.rs`.

## 📚 Additional Resources

- **Architecture Details**: See `docs/ARCHITECTURE.md`
- **Config Reference**: See `docs/CONFIGURATION.md`
- **Tauri Docs**: https://tauri.app/
- **Vue 3 Docs**: https://vuejs.org/