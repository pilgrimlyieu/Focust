# Focust Architecture Documentation

This document provides a comprehensive overview of Focust's architecture, covering the technology stack, system design, module organization, data flow, and core logic.

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [System Architecture](#system-architecture)
- [Backend Architecture (Rust)](#backend-architecture-rust)
- [Frontend Architecture (Vue 3)](#frontend-architecture-vue-3)
- [Core Logic & Data Flow](#core-logic--data-flow)
- [Module Details](#module-details)
- [Testing Strategy](#testing-strategy)
- [Performance Considerations](#performance-considerations)

---

## Overview

Focust is a cross-platform break reminder application built with **Tauri 2** (Rust backend) and **Vue 3** (TypeScript frontend). The application follows a clean separation between:

- **Backend (Rust)**: Core logic, scheduling engine, system integrations, configuration management
- **Frontend (Vue 3)**: User interface, state management, visual presentation
- **IPC Bridge**: Tauri's command system for bidirectional communication

### Design Philosophy

1. **Event-Driven Architecture**: Scheduler uses an event-driven model for break management
2. **Type Safety**: End-to-end type safety from Rust to TypeScript via ts-rs
3. **Modular Design**: Clear separation of concerns with well-defined module boundaries
4. **Platform Integration**: Native system features (tray, hotkeys, notifications, idle detection)
5. **Configuration-First**: TOML-based configuration with sensible defaults

---

## Technology Stack

### Backend (Rust)

| Component | Version | Purpose |
|-----------|---------|---------|
| **Rust** | 2024 Edition | Systems programming language |
| **Tauri** | 2.x | Cross-platform desktop framework |
| **Tokio** | 1.x | Async runtime |
| **Serde** | 1.x | Serialization/deserialization |
| **ts-rs** | 11.x | TypeScript type generation |
| **Rodio** | 0.21 | Audio playback |
| **user-idle** | 0.6 | System idle detection |
| **tracing** | 0.1 | Logging and diagnostics |
| **chrono** | 0.4 | Date and time handling |

### Frontend (TypeScript/Vue)

| Component | Version | Purpose |
|-----------|---------|---------|
| **Vue** | 3.5 | Progressive JavaScript framework |
| **TypeScript** | 5.9 | Type-safe JavaScript |
| **Pinia** | 3.x | State management |
| **Vue I18n** | 11.x | Internationalization |
| **Vite** | 7.x | Build tool and dev server |
| **Tailwind CSS** | 4.x | Utility-first CSS framework |
| **DaisyUI** | 5.x | Component library |
| **Vitest** | 4.x | Unit testing |

### Development Tools

| Tool | Purpose |
|------|---------|
| **Just** | Command runner (like Make) |
| **Biome** | Code formatter and linter |
| **Cargo** | Rust package manager |
| **Bun/npm** | JavaScript package manager |

---

## System Architecture

### High-Level Architecture

```
┌────────────────────────────────────────────────────────────┐
│                     User Interface (Vue 3)                 │
│  ┌───────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │   Settings    │  │  Break Window │  │  Toast/Modals  │  │
│  │    Window     │  │   (Dynamic)   │  │                │  │
│  └───────┬───────┘  └───────┬───────┘  └────────────────┘  │
│          │                  │                              │
│  ┌───────▼──────────────────▼──────────────────────────┐   │
│  │            Pinia State Management                   │   │
│  │  (configStore, schedulerStore, suggestionsStore)    │   │
│  └────────────────────┬────────────────────────────────┘   │
└───────────────────────┼────────────────────────────────────┘
                        │
                        │ Tauri IPC (Commands & Events)
                        │
┌───────────────────────▼────────────────────────────────────┐
│                  Tauri Backend (Rust)                      │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              Command Handlers (cmd/)                  │ │
│  │  • config   • scheduler   • audio   • window   • sys  │ │
│  └─────────────────┬─────────────────────────────────────┘ │
│                    │                                       │
│  ┌─────────────────▼─────────────┬─────────────────────┐   │
│  │   Scheduler Engine            │  Configuration      │   │
│  │   (scheduler/)                │  (config/)          │   │
│  │  • Event-driven loop          │  • TOML storage     │   │
│  │  • State machine              │  • Type-safe models │   │
│  │  • Event sources              │  • Default values   │   │
│  └────┬──────────────────────────┴─────────────────────┘   │
│       │                                                    │
│  ┌────▼──────────────────┐  ┌──────────────────────────┐   │
│  │  Core Business Logic  │  │  Platform Integration    │   │
│  │  (core/)              │  │  (platform/)             │   │
│  │  • Audio playback     │  │  • System tray           │   │
│  │  • Break scheduling   │  │  • Global hotkeys        │   │
│  │  • Suggestions        │  │  • Notifications         │   │
│  │  • Theme management   │  │  • Idle detection        │   │
│  └───────────────────────┘  └──────────────────────────┘   │
└────────────────────────────────────────────────────────────┘
```

### Communication Patterns

1. **Frontend → Backend**: Tauri Commands (async/await)
2. **Backend → Frontend**: Tauri Events (publish/subscribe)
3. **State Persistence**: TOML files in platform-specific config directory
4. **Type Safety**: Rust types exported to TypeScript via ts-rs

---

## Backend Architecture (Rust)

### Module Organization

```
src-tauri/src/
├── lib.rs                      # Library root, app setup
├── main.rs                     # Entry point
│
├── cmd/                        # Tauri command handlers
│   ├── mod.rs
│   ├── audio.rs                # Audio playback commands
│   ├── config.rs               # Config load/save commands
│   ├── payload.rs              # Break payload management
│   ├── scheduler.rs            # Scheduler control commands
│   ├── suggestions.rs          # Suggestions commands
│   ├── system.rs               # System utilities
│   └── window.rs               # Window management
│
├── config/                     # Configuration system
│   ├── mod.rs
│   ├── core.rs                 # Load/save logic
│   └── models.rs               # AppConfig and SharedConfig
│
├── core/                       # Business logic
│   ├── mod.rs
│   ├── schedule.rs             # Break schedule types
│   ├── suggestions.rs          # Suggestion system
│   ├── theme.rs                # Theme configuration types
│   ├── time.rs                 # Time utilities
│   └── audio/                  # Audio subsystem
│       ├── mod.rs
│       ├── models.rs           # Audio types
│       └── player.rs           # Rodio-based player
│
├── scheduler/                  # Scheduling engine
│   ├── mod.rs
│   ├── core.rs                 # Main scheduler loop
│   ├── event.rs                # Event sources
│   └── models.rs               # Scheduler types
│
├── platform/                   # Platform integrations
│   ├── mod.rs
│   ├── tray.rs                 # System tray
│   ├── hotkey.rs               # Global shortcuts
│   └── notifications.rs        # System notifications
│
└── utils/                      # Utilities
    ├── mod.rs
    └── logging.rs              # Tracing setup
```

### Data Flow (Backend)

1. **App Startup:**
   ```
   main.rs
     → load_config() → AppConfig
     → init_audio_player() → AudioPlayer
     → setup_tray() → TrayIcon
     → register_shortcuts() → GlobalShortcut
     → init_scheduler() → Scheduler loop starts
   ```

2. **Break Triggering:**
   ```
   Scheduler calculates next event
     → Waits until event time
     → Creates BreakPayload
     → Stores in BreakPayloadStore (by timestamp UUID)
     → Emits "show-break" event to frontend
     → Frontend opens break window with payload ID
     → Frontend fetches payload via command
     → Displays break UI
   ```

3. **Config Update:**
   ```
   Frontend calls save_config(config)
     → Validate config
     → Serialize to TOML
     → Write to config file
     → Update SharedConfig
     → Emit "config-updated" event
     → Scheduler picks up changes on next cycle
   ```

---

## Frontend Architecture (Vue 3)

### Module Organization

```
src/
├── main.ts                     # App entry point
├── settings.ts                 # Settings window entry
├── App.vue                     # Root component (unused, for future)
│
├── views/                      # Page-level components
│   ├── SettingsApp.vue         # Settings window
│   └── BreakApp.vue            # Break window
│
├── components/
│   ├── settings/               # Settings panel components
│   │   ├── GeneralSettingsPanel.vue
│   │   ├── SchedulesPanel.vue
│   │   ├── AttentionsPanel.vue
│   │   ├── SuggestionsPanel.vue
│   │   └── AdvancedPanel.vue
│   ├── ui/                     # Reusable UI components
│   │   ├── ToastHost.vue
│   │   └── ...
│   └── icons/                  # Icon components
│
├── stores/                     # Pinia state management
│   ├── config.ts               # Configuration state
│   ├── scheduler.ts            # Scheduler state
│   └── suggestions.ts          # Suggestions state
│
├── composables/                # Composition API utilities
│   ├── useComputed.ts          # Custom computed properties
│   └── useToast.ts             # Toast notification system
│
├── types/
│   ├── guards.ts               # Type guard functions
│   ├── factories.ts            # Type factory functions
│   └── generated/              # Auto-generated from Rust
│
├── i18n/                       # Internationalization
│   ├── index.ts
│   └── locales/
│       ├── en-US.ts
│       └── zh-CN.ts
│
└── utils/                      # Utility functions
    ├── handleError.ts
    └── safeClone.ts
```


### Views

**Settings Window (`views/SettingsApp.vue`):**
- Dynamical creation and destroy for low RAM usage in background
- Tab-based navigation (General, Schedules, Attentions, Suggestions, Advanced)
- Real-time scheduler status display
- Save/Reset/Pause/Postpone controls
- Lazy-loaded panel components for performance

**Break Window (`views/BreakApp.vue`):**
- Fetches break payload by ID from URL query parameter
- Displays countdown timer with circular progress
- Shows suggestion (if enabled)
- Handles keyboard shortcuts (Enter to finish, configurable key to postpone)
- Plays audio (only on primary window in multi-monitor setup)
- Prevents closing/skipping in strict mode

### Type Safety

**Generated Types:**
Rust types are automatically exported to TypeScript using ts-rs:

```rust
// Rust (src-tauri/src/config/models.rs)
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig {
    pub check_for_updates: bool,
    // ...
}
```

```typescript
// TypeScript (src/types/generated/AppConfig.ts)
export interface AppConfig {
  checkForUpdates: boolean;
  // ...
}
```
### Data Flow (Frontend)

1. **Settings Load:**
   ```
   SettingsApp.vue mounted
     → configStore.load()
     → invoke("get_config")
     → Update draft and original
   ```

2. **Config Save:**
   ```
   User clicks Save
     → configStore.save()
     → invoke("save_config", { config })
     → Backend saves to TOML
     → Update original = draft
   ```

3. **Break Display:**
   ```
   Backend emits "show-break" event
     → Frontend receives payloadId
     → Create new window with URL: /break?payloadId=...
     → BreakApp.vue mounted
     → invoke("get_break_payload", { payloadId })
     → Display break UI
     → Start countdown timer
     → Play audio (if primary window)
   ```

4. **Real-Time Status:**
   ```
   Scheduler calculates next event
     → Emits "scheduler-status" event
     → schedulerStore receives update
     → SettingsApp displays next break time
     → Live countdown updated every second
   ```

---

## Core Logic & Data Flow

### Complete Break Flow

```
┌───────────────────────────────────────────────────────────────┐
│  1. Scheduler Calculates Next Event                           │
│     • Checks all active schedules and attentions              │
│     • Considers time ranges, days of week                     │
│     • Accounts for mini/long break cycle                      │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  2. Wait Until Event Time                                     │
│     • tokio::select! on:                                      │
│       - Sleep until event                                     │
│       - Command channel (pause, postpone, etc.)               │
│       - Shutdown signal                                       │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  3. Notification (if configured)                              │
│     • Wait until (event_time - notification_before_s)         │
│     • Send system notification                                │
│     • Continue waiting for actual break                       │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  4. Create Break Payload                                      │
│     • Load schedule/attention settings                        │
│     • Resolve background image path                           │
│     • Pick random suggestion (if enabled)                     │
│     • Create BreakPayload struct                              │
│     • Generate UUID                                           │
│     • Store in BreakPayloadStore                              │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  5. Create Break Windows                                      │
│     • Get all monitors (if all_screens enabled)               │
│     • For each monitor:                                       │
│       - Create window with label "break-<monitor_index>"      │
│       - Set URL: /break?payloadId=<uuid>                      │
│       - Configure size based on window_size                   │
│       - Set decorations, always_on_top, etc.                  │
│     • Emit "show-break" event                                 │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  6. Frontend Displays Break                                   │
│     • BreakApp.vue receives payloadId from URL                │
│     • Fetches payload: invoke("get_break_payload")            │
│     • Preloads background image                               │
│     • Starts countdown timer (updates every 1s)               │
│     • Plays audio (primary window only)                       │
│     • Shows window and brings to focus                        │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  7. User Interaction                                          │
│     • Wait for:                                               │
│       - Timer expires (auto-finish)                           │
│       - User clicks "Resume" button                           │
│       - User clicks "Postpone" button                         │
│       - User presses Enter (finish)                           │
│       - User presses postpone shortcut key                    │
│     • Strict mode: Only auto-finish allowed                   │
└─────────────────────┬─────────────────────────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────────────────────────┐
│  8. Break Finish/Postpone                                     │
│     • Finish:                                                 │
│       - Stop audio                                            │
│       - Emit "break-finished" event                           │
│       - Close all break windows                               │
│       - Scheduler continues to next event                     │
│     • Postpone:                                               │
│       - invoke("postpone_break")                              │
│       - Scheduler delays next break by postponed_s            │
│       - Close windows                                         │
└───────────────────────────────────────────────────────────────┘
```

### Idle Detection Flow

```
┌───────────────────────────────────────────────────────────────┐
│  Idle Detection Thread (runs continuously)                    │
└────────────────┬──────────────────────────────────────────────┘
                 │
                 ▼
       ┌─────────────────────┐
       │  Check idle time    │
       │  every 5 seconds    │
       └──────────┬──────────┘
                  │
          ┌───────▼───────┐
          │ idle_time >   │
          │ inactive_s ?  │
          └───┬───────┬───┘
              │       │
         Yes  │       │  No
              │       └──────────────┐
              ▼                      │
    ┌───────────────────┐            │
    │ was_idle == false │            │
    └────┬──────────────┘            │
         │ Yes                       │
         ▼                           │
  ┌────────────────────┐             │
  │ Send Pause command │             │
  │ to scheduler       │             │
  │ was_idle = true    │             │
  └──────────┬─────────┘             │
             │                       │
             └──────────┬────────────┘
                        │
                        ▼
                ┌───────────────┐
                │ Continue loop │
                └───────────────┘
```

---

## Module Details

### Break Payload Structure

```typescript
interface BreakPayload {
  id: number;
  kind: "mini" | "long" | "attention";
  title: string;
  message: string | null;
  messageKey: string;
  scheduleName: string | null;
  duration: number;  // seconds
  strictMode: boolean;
  postponeShortcut: string;
  suggestion: string | null;
  theme: ThemeSettings;
  audio: AudioSettings | null;
  background: {
    type: "solid" | "image";
    value: string;  // hex color or image path
  };
}
```

### Theme Resolution

When displaying a break window:

1. **Background:**
   - `Solid`: Use hex color directly
   - `ImagePath`: Convert to `asset://localhost/` URL
   - `ImageFolder`: (planned) Pick random image from folder

2. **Text Color:** Applied to all text elements

3. **Blur & Opacity:** Applied to backdrop overlay

4. **Font:** Set via CSS font-family

### Audio Playback

**Built-in Sounds:**
Located in `src-tauri/assets/sounds/`:
- `gentle-bell.mp3`
- `soft-gong.mp3`
- `bright-notification.mp3`
- `notification.mp3`

**Loading:**
```rust
// Built-in
let resource_path = app_handle
    .path()
    .resolve("assets/sounds/{name}.mp3", BaseDirectory::Resource)?;

// Custom file
let path = user_provided_path;
```

**Playback:**
- Only plays in primary window (monitor index 0)
- Stops when break finishes/is postponed
- Volume range: 0.0 - 1.0

---

## Testing Strategy

### Backend Tests

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.language, parsed.language);
    }
}
```

**Integration Tests:**
Located in `src-tauri/tests/`:
- `config_serialization_test.rs` - Config load/save
- `scheduler_integration_test.rs` - Scheduler logic
- `comprehensive_integration_test.rs` - Full system tests

**Running Tests:**
```bash
cargo test --workspace          # All tests
cargo test config               # Specific module
cargo test --test scheduler     # Specific test file
```

### Frontend Tests

**Unit Tests:**
```typescript
import { describe, it, expect } from "vitest";
import { useConfigStore } from "@/stores/config";

describe("configStore", () => {
  it("detects dirty state", () => {
    const store = useConfigStore();
    store.draft = { ...store.original, language: "zh-CN" };
    expect(store.isDirty).toBe(true);
  });
});
```

**Component Tests:**
```typescript
import { mount } from "@vue/test-utils";
import ToastHost from "@/components/ui/ToastHost.vue";

describe("ToastHost", () => {
  it("renders toast messages", () => {
    const wrapper = mount(ToastHost, {
      props: {
        toasts: [{ id: "1", kind: "success", message: "Saved!" }],
      },
    });
    expect(wrapper.text()).toContain("Saved!");
  });
});
```

**Running Tests:**
```bash
bun run test:run                # All tests
bun run test:ui                 # With UI
bun run test:coverage           # With coverage
```

---

## Performance Considerations

### Backend

1. **Async/Await**: All I/O operations are async to prevent blocking
2. **Lazy Loading**: Audio player initialized only when needed
3. **Efficient Scheduling**: Scheduler sleeps until next event (no polling)
4. **Shared State**: `RwLock` for concurrent reads, exclusive writes
5. **Resource Cleanup**: Proper drop implementations for audio, windows

### Frontend

1. **Code Splitting**: Settings panels loaded lazily with `defineAsyncComponent`
2. **Reactive Updates**: Vue's reactivity system ensures minimal re-renders
3. **Debouncing**: User input debounced where appropriate
4. **Image Preloading**: Background images preloaded before showing window
5. **Memoization**: Computed properties cache results

### Window Management

1. **On-Demand Creation**: Break windows created only when needed
2. **Proper Cleanup**: Windows destroyed after break finishes
3. **Resource Pooling**: (potential improvement) Pre-create window pool

---

## Debugging and Logging

### Backend Logging

```rust
// Logging levels: trace, debug, info, warn, error
tracing::debug!("Config loaded: {:?}", config);
tracing::info!("Scheduler started");
tracing::warn!("No active schedule found");
tracing::error!("Failed to play audio: {}", err);
```

**Log Files:**
- Location: Platform-specific app log directory
- File: `focust.log` with date-based rotation
- Rotation: Daily

### Frontend Debugging

```typescript
// Console logging
console.log("[Store] Config loaded:", config);
console.warn("[Scheduler] Status update delayed");

// Vue Devtools
// Install browser extension for reactive state inspection
```

---

For more information:
- [CONFIGURATION.md](CONFIGURATION.md) - Configuration reference
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guide
- [README.md](../README.md) - Project overview
