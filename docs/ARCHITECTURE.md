# Focust Architecture Documentation

This document provides a high-level overview of Focust's architecture. For detailed implementation, please refer to the source code.

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [System Architecture](#system-architecture)
- [Backend Architecture](#backend-architecture)
- [Frontend Architecture](#frontend-architecture)
- [Key Features](#key-features)
- [Development Guidelines](#development-guidelines)

---

## Overview

Focust is a cross-platform break reminder application built with **Tauri 2** (Rust backend) and **Vue 3** (TypeScript frontend).

### Design Philosophy

1. **Event-Driven**: Scheduler uses events for break management
2. **Type Safety**: End-to-end type safety via ts-rs
3. **Modular**: Clear separation of concerns
4. **Native Integration**: System tray, hotkeys, notifications
5. **Configuration-First**: TOML-based with sensible defaults

---

## Technology Stack

### Backend (Rust)

- **Tauri 2.x** - Desktop framework
- **Tokio** - Async runtime
- **Serde + TOML** - Config serialization
- **ts-rs** - TypeScript type generation
- **user-idle** - System idle detection
- **tracing** - Logging

### Frontend (Vue 3)

- **Vue 3.5** + **TypeScript 5.9**
- **Pinia** - State management
- **Vue I18n** - Internationalization
- **Tailwind CSS + DaisyUI** - UI styling
- **Vite 7** - Build tool
- **Vitest** - Testing

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────┐
│         Frontend (Vue 3)                    │
│  Settings Window  |  Break Windows          │
│  Pinia Stores     |  Toast/Modal            │
└────────────┬────────────────────────────────┘
             │ Tauri IPC (Commands & Events)
┌────────────┴────────────────────────────────┐
│         Backend (Rust)                      │
│  ┌──────────────┬────────────────────────┐  │
│  │ Commands     │ Configuration          │  │
│  ├──────────────┼────────────────────────┤  │
│  │ Scheduler    │ Platform Integration   │  │
│  │ Audio/Theme  │ Tray/Hotkeys/Idle      │  │
│  └──────────────┴────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### Communication Patterns

- **Frontend → Backend**: Tauri commands (`invoke()`)
- **Backend → Frontend**: Tauri events (`emit()`)
- **Persistence**: TOML files in platform config directory
- **Type Safety**: Rust types → TypeScript via ts-rs

---

## Backend Architecture

### Module Structure

```
src-tauri/src/
├── cmd/                 # Tauri command handlers
├── config/              # Configuration system
│   ├── core.rs          # Load/save with partial config support
│   └── models.rs        # AppConfig type definitions
├── core/                # Business logic
│   ├── audio/           # Audio playback (Rodio)
│   ├── schedule.rs      # Break schedule types
│   ├── suggestions.rs   # Suggestion system
│   ├── theme.rs         # Theme types
│   └── time.rs          # Time utilities
├── scheduler/           # Scheduling engine
│   ├── core.rs          # Event-driven scheduler loop
│   ├── event.rs         # Event source calculations
│   └── models.rs        # Scheduler state types
├── platform/            # Platform integrations
│   ├── tray.rs          # System tray
│   ├── hotkey.rs        # Global shortcuts
│   ├── i18n.rs          # Internationalization
│   └── notifications.rs # System notifications
└── utils/               # Utilities
    └── logging.rs       # Tracing setup
```

### Key Concepts

**Configuration System:**
- TOML-based with **partial config loading**
- Missing fields use default values
- Automatic config migration on version updates
- Type-safe with `serde` + `ts-rs`

**Scheduler Engine:**
- Event-driven state machine
- States: `Running`, `Paused`, `Idle`, `PostBreak`
- Event sources: Mini breaks, long breaks, attention reminders
- Auto-pause on system idle

**Break Payload:**
- Created when break triggers
- Stored by UUID in `PromptPayloadStore`
- Frontend fetches via command with UUID

---

## Frontend Architecture

### Module Structure

```
src/
├── views/              # Main views
│   ├── SettingsApp.vue # Settings window
│   └── PromptApp.vue    # Break window
├── components/
│   ├── settings/       # Settings panels (lazy-loaded)
│   ├── ui/             # Reusable components
│   └── icons/          # Icon components
├── stores/             # Pinia stores
│   ├── config.ts       # Config state + dirty tracking
│   ├── scheduler.ts    # Scheduler status
│   └── suggestions.ts  # Suggestion management
├── composables/        # Composition utilities
│   ├── useComputed.ts  # Custom computed
│   └── useToast.ts     # Toast notifications
├── types/
│   ├── generated/      # Auto-generated from Rust
│   ├── guards.ts       # Type guards
│   └── factories.ts    # Factory functions
├── i18n/               # Internationalization
└── utils/              # Utility functions
```

### Key Features

**Settings Window:**
- Tab-based UI (General, Schedules, Attentions, etc.)
- Dirty state tracking for unsaved changes
- Lazy-loaded panels for performance
- Real-time scheduler status display

**Break Window:**
- Dynamic creation per monitor
- Countdown timer with circular progress
- Suggestion display (if enabled)
- Audio playback (primary window only)
- Keyboard shortcuts (Enter/postpone key)

**State Management:**
- `configStore`: Config CRUD + dirty tracking
- `schedulerStore`: Scheduler status + next break time
- `suggestionsStore`: Suggestion CRUD

---

## Key Features

### 1. Partial Config Loading

**Problem:** New app versions add fields → old configs fail to parse

**Solution:** Merge valid fields with defaults

```rust
// If config parse fails:
// 1. Parse as generic TOML
// 2. Extract valid fields
// 3. Fill missing fields with defaults
// 4. Save merged config
```

**Benefits:**
- Seamless version upgrades
- No data loss
- User settings preserved

### 2. Event-Driven Scheduler

**Flow:**
```
Calculate next event
  ↓
Wait until event (tokio::select!)
  ↓
Send notification (if configured)
  ↓
Create prompt payload
  ↓
Emit "show-prompt" event
  ↓
Frontend creates break windows
  ↓
User finishes/postpones
  ↓
Loop
```

**Features:**
- Non-blocking async design
- Handles pause/resume/postpone commands
- Auto-pause on idle detection
- Multi-monitor support

### 3. Type-Safe IPC

**Rust → TypeScript:**
```rust
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    pub autostart: bool,
    // ...
}
```

Generated TypeScript:
```typescript
export interface AppConfig {
  autostart: boolean;
  // ...
}
```

**Benefits:**
- Compile-time type checking
- Refactoring safety
- IDE autocomplete

### 4. Multi-Monitor Break Windows

**Implementation:**
```rust
// Get all monitors
let monitors = app_handle.available_monitors()?;

// Create window per monitor
for (index, monitor) in monitors.iter().enumerate() {
    let window = create_break_window(
        app_handle,
        &format!("break-{}", index),
        payload_id,
        monitor
    )?;
}
```

**Audio:** Only plays in primary window (index 0)

### 5. Idle Detection & Auto-Pause

**Thread-based monitoring:**
```rust
// Check idle time every 5 seconds
loop {
    let idle_time = user_idle::UserIdle::get_time()?;
    
    if idle_time > inactive_s {
        // Send pause command
    }
    
    sleep(Duration::from_secs(10));
}
```

**Seamless resume:** Automatically resumes when activity detected

---

## Development Guidelines

### Adding New Config Fields

1. **Add field to `AppConfig`** in `config/models.rs`:
   ```rust
   pub struct AppConfig {
       pub new_field: bool,
   }
   ```

2. **Update `Default` impl** with default value

3. **Update `merge_config_field()`** in `config/core.rs`:
   ```rust
   merge_field!(new_field, "newField", bool);
   ```

4. **Run `cargo test export_bindings`** to generate TypeScript types

5. **Update frontend** to use new field

### Adding New Tauri Commands

1. **Create command** in `cmd/`:
   ```rust
   #[tauri::command]
   pub async fn my_command(arg: String) -> Result<String, String> {
       Ok(format!("Result: {arg}"))
   }
   ```

2. **Register in `lib.rs`**:
   ```rust
   .invoke_handler(tauri::generate_handler![
       my_command,
   ])
   ```

3. **Add permission** in `capabilities/default.json`:
   ```json
   {
     "identifier": "my_command",
     "allow": ["my_command"]
   }
   ```

4. **Call from frontend**:
   ```typescript
   import { invoke } from '@tauri-apps/api/core';
   const result = await invoke<string>('my_command', { arg: 'test' });
   ```

### Testing

**Backend:**
```bash
just test-back-all     # All Rust tests
just test-back <name>  # Specific test
```

**Frontend:**
```bash
just test-front-all    # All Vue tests
just test-front <name> # Specific test
```

**Integration:**
```bash
just test-all          # Everything
```

---

## Additional Resources

- **[CONFIGURATION.md](CONFIGURATION.md)** - Config file reference
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Development guide
