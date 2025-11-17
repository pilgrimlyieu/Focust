# Focust Architecture Overview

> **Purpose**: One-page system overview for developers  
> **Audience**: Developers joining the project  
> **Detail Level**: Architecture + modules, no implementation

---

## System Architecture

### High-Level Diagram

```
┌─────────────── Frontend (Vue 3 + TypeScript) ───────────────┐
│  Settings Window              Break Windows (Multi-Monitor) │
│  ├─ Pinia Stores              ├─ Countdown Timer            │
│  ├─ Settings Panels           ├─ Suggestion Display         │
│  └─ Status Display            └─ Audio Playback             │
└────────────────────┬────────────────────────────────────────┘
                     │ Tauri IPC (23 Commands + 5 Events)
┌────────────────────┴─── Backend (Rust + Tauri 2) ───────────┐
│  Scheduler Manager          Platform Integration            │
│  ├─ BreakScheduler          ├─ System Tray                  │
│  ├─ AttentionTimer          ├─ Global Hotkeys               │
│  └─ SharedState             ├─ Idle Detection               │
│                             └─ DND Monitoring               │
│  Config System              Core                            │
│  ├─ TOML Load/Save          ├─ Audio (Rodio)                │
│  └─ Partial Merge           ├─ Themes                       │
│                             └─ Suggestions                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Breakdown

### Backend (Rust) - `src-tauri/src/`

| Module         | Purpose                              | Key Components                                                   |
| -------------- | ------------------------------------ | ---------------------------------------------------------------- |
| **cmd/**       | Tauri command handlers (23 commands) | audio, autostart, config, scheduler, suggestions, system, window |
| **config/**    | TOML config with partial loading     | AppConfig, AdvancedSettings, load/save/merge                     |
| **scheduler/** | Event-driven scheduling engine       | BreakScheduler, AttentionTimer, SharedState, Command routing     |
| **monitors/**  | Environment monitoring               | IdleMonitor, DndMonitor, AppWhitelistMonitor, Orchestrator       |
| **platform/**  | System integration                   | Tray, hotkeys, notifications, i18n, window creation              |
| **core/**      | Business logic types                 | Audio, Schedule, Suggestions, Theme, Time                        |
| **utils/**     | Utilities                            | Error handling, logging (tracing)                                |

### Frontend (Vue 3) - `src/`

| Module           | Purpose                   | Key Components                                |
| ---------------- | ------------------------- | --------------------------------------------- |
| **views/**       | Main application views    | SettingsApp.vue, PromptApp.vue                |
| **stores/**      | Pinia state management    | configStore, schedulerStore, suggestionsStore |
| **components/**  | Reusable components       | settings/ (lazy panels), ui/, icons/          |
| **types/**       | Type system               | generated/ (ts-rs), guards.ts, factories.ts   |
| **composables/** | Vue composition functions | useToast, useComputed                         |
| **i18n/**        | Internationalization      | en-US, zh-CN locales                          |
| **utils/**       | Utility functions         | safeClone, handleError                        |

---

## Data Flow

### 1. Configuration Flow

```
User Edit (Settings UI)
  ↓
configStore.draft (Pinia)
  ↓
invoke("save_config")
  ↓
Backend: save_config() → TOML file
  ↓
SchedulerManager: UpdateConfig command
  ↓
BreakScheduler + AttentionTimer recalculate
```

### 2. Scheduler Flow

```
BreakScheduler: Calculate next break
  ↓
Wait until break time (tokio::select!)
  ↓
Send notification (optional)
  ↓
Execute break: emit("scheduler-event", payload)
  ↓
Frontend: schedulerStore.handleSchedulerEvent()
  ↓
Create break windows (one per monitor)
  ↓
User finishes → invoke("break_finished")
  ↓
BreakScheduler: Update state, calculate next
```

### 3. Monitor Flow

```
Orchestrator: Sequential monitor checking
  ↓
IdleMonitor: Check system idle time
DndMonitor: Check Do Not Disturb state
AppWhitelistMonitor: Check active processes
  ↓
Convert MonitorResult → Pause/Resume reason
  ↓
SchedulerManager: Add/Remove pause reason (bitflags)
  ↓
If all clear → Resume schedulers
If any active → Pause schedulers
```

---

## IPC Communication

### Commands (Frontend → Backend)

| Command                                                                                                                               | Module      | Purpose                   |
| ------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------- |
| `get_config`, `save_config`, `pick_background_image`                                                                                  | config      | Config CRUD               |
| `get_autostart`, `set_autostart`                                                                                                      | autostart   | Autostart management      |
| `play_builtin_audio`, `play_audio_file`, `stop_audio`                                                                                 | audio       | Audio control             |
| `pause_scheduler`, `resume_scheduler`, `postpone_break`, `skip_break`, `trigger_event`, `request_scheduler_status`, `reset_scheduler` | scheduler   | Scheduler control         |
| `get_suggestions`, `save_suggestions`, `get_suggestions_for_language`                                                                 | suggestions | Suggestion CRUD           |
| `get_payload`, `cleanup_payload`                                                                                                      | payload     | Prompt payload management |
| `open_config_dir`, `open_log_dir`, `open_folder`                                                                                      | system      | File explorer             |
| `open_settings_window`, `close_break_windows`                                                                                         | window      | Window management         |

### Events (Backend → Frontend)

| Event                    | Payload           | Purpose                             |
| ------------------------ | ----------------- | ----------------------------------- |
| `scheduler-status`       | `SchedulerStatus` | Status updates (paused, next event) |
| `scheduler-event`        | `PromptPayload`   | Break/attention trigger             |
| `scheduler-paused`       | `()`              | Paused notification                 |
| `scheduler-resumed`      | `()`              | Resumed notification                |
| `postpone-limit-reached` | `()`              | Max postpones reached               |

---

## Key Design Patterns

### 1. Dual Scheduler Architecture

**BreakScheduler**: Handles mini/long breaks with intervals  
**AttentionTimer**: Handles time-based attention reminders  
**SharedState**: Central authority for pause reasons (bitflags)

**Why**: Separate concerns, independent timing logic, unified pause management

### 2. Bitflags for Pause Reasons

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

**Why**: Multiple pause reasons coexist, resume only when all cleared

### 3. Partial Config Loading

**Problem**: New app versions add fields → old configs break  
**Solution**: Parse as generic TOML, merge valid fields with defaults  
**Why**: Seamless upgrades, no data loss, backward compatibility

### 4. Event-Driven Monitoring

**Windows/Linux**: DND monitor uses platform events (WNF/D-Bus)  
**macOS**: Polling (no native event API)  
**Why**: Minimize CPU usage, responsive state changes

### 5. Type-Safe IPC (ts-rs)

```rust
#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct AppConfig { }
```

**Why**: Compile-time type checking, refactoring safety, IDE support

### 6. Session Protection

**SharedState**: Tracks break/attention sessions  
**Monitors**: Check `in_any_session()` before acting  
**Why**: Prevent monitors from interfering with active prompts

---

## Technology Choices

| Decision         | Choice                   | Rationale                            |
| ---------------- | ------------------------ | ------------------------------------ |
| Config Format    | TOML                     | Human-readable, comments, type-safe  |
| State Management | Pinia                    | Vue 3 official, TypeScript support   |
| Scheduling       | Event-driven             | Low CPU, responsive, testable        |
| Error Handling   | `Result<T,E>` + `anyhow` | Library uses Result, app uses anyhow |
| Async Runtime    | `tauri::async_runtime`   | Avoid Tokio Runtime errors           |
| Type Sync        | ts-rs                    | Auto-generate TS types from Rust     |
| Logging          | tracing                  | Structured, file output, levels      |
| Audio            | rodio                    | Cross-platform, multi-format         |
| Testing          | Vitest + Cargo test      | Native support, ecosystem            |

---

## Testing Strategy

### Test Layers

```
Layer 4: End-to-End (User scenarios)
Layer 3: Integration (Manager + Monitor coordination)
Layer 2: State Machine (BreakScheduler + AttentionTimer logic)
Layer 1: Unit (SharedState + Config + Core types)
```

**Coverage**: 273 tests, 100% pass rate  
**Key Tests**: 143 scheduler tests (8 stress tests), 30 config tests

---

## Platform Support

| Platform | Status         | Notes                                   |
| -------- | -------------- | --------------------------------------- |
| Windows  | ✅ Fully tested | Primary development platform            |
| Linux    | ✅ Designed     | Event-driven DND (D-Bus), tested in CI  |
| macOS    | ⚠️ Partial      | Audio disabled (cpal Send), DND polling |

**Platform-Specific Code**: `#[cfg(target_os = "...")]` in `platform/dnd/`

---

## Resources

- **Config Reference**: `docs/CONFIGURATION.md`
