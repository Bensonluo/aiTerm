# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Install dependencies
pnpm install

# Development
pnpm tauri dev          # Start Tauri dev server with hot reload
pnpm dev                # Vue frontend only (port 1420)

# Build
pnpm tauri build        # Production build (outputs to src-tauri/target/release/)
pnpm build              # Vue frontend build only

# Linting & Formatting
pnpm lint               # ESLint with auto-fix
pnpm format             # Prettier formatting
pnpm lint:rust          # Rust clippy (cd src-tauri && cargo clippy -- -D warnings)

# Type checking
pnpm build              # Runs vue-tsc --noEmit && vite build
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  Vue 3 Frontend (src/)          Tauri IPC Events/Invoke     │
│  ├── App.vue (orchestration)     ←────────────────────────►│
│  ├── Terminal.vue (xterm.js)     PTY, AI, Context Commands │
│  ├── AIChat.vue (AI panel)                                 │
│  └── Stores (Pinia)                                        │
│      ├── terminal.ts                                        │
│      ├── ai.ts                                             │
│      └── settings.ts                                       │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (src-tauri/src/)                              │
│  ├── lib.rs (Tauri commands)    AppState (Mutex-protected)   │
│  ├── pty/mod.rs                PTY session management       │
│  ├── llm/mod.rs                Ollama/OpenAI client         │
│  ├── context/mod.rs            Ring buffer + summarization  │
│  └── config/mod.rs             Persistent config (~/.aiterm)│
└─────────────────────────────────────────────────────────────┘
```

### Frontend Structure (src/)
- **App.vue** - Main orchestrator: PTY lifecycle, event listeners, AI command execution
- **components/Terminal.vue** - xterm.js wrapper with fit addon
- **components/AIChat.vue** - AI chat panel with streaming responses
- **components/Settings.vue** - Settings modal (provider config, terminal options)
- **components/CmdConfirm.vue** - Command confirmation dialog for AI-suggested commands
- **stores/** - Pinia stores for terminal state, AI messages, and settings

### Backend Modules (src-tauri/src/)
- **pty/mod.rs** - Spawns shell processes via `portable-pty`, emits `pty_output`/`pty_exit` events
- **llm/mod.rs** - Unified LLM client for Ollama and OpenAI-compatible APIs; streaming responses
- **context/mod.rs** - Ring buffer (500 lines default) with LLM summarization for long sessions
- **config/mod.rs** - Persistent JSON config in `~/.aiterm/config.json`

### Key Tauri Commands
```
PTY:     pty_create, pty_write, pty_resize, pty_destroy, pty_exists
AI:      ai_set_provider, ai_list_models, ai_test_connection, ai_chat
Context: context_push_input, context_push_output, context_stats, context_summarize
Settings: get_settings, save_settings
```

## Key Patterns

### Tauri Event Flow
1. Frontend calls `invoke('pty_create', {...})` to spawn PTY
2. Backend spawns shell in background thread, emits `pty_output` events
3. Frontend listens for `pty_output` via `listen()`, writes to xterm.js

### AI Streaming
1. Frontend calls `invoke('ai_chat', {message})`
2. Backend streams via `app_handle.emit('ai_stream', {content})`
3. Frontend accumulates chunks in `AIChat.vue` via `listen('ai_stream')`

### Context Management
- Ring buffer (500 lines) stores recent terminal I/O
- When buffer overflows, entries move to "pending summary" pool
- When 100+ pending entries, `context_summarize` LLM call compresses them
- AI context = summary (70% token budget) + recent entries (30%)

### State Synchronization
AppState in Rust holds `Mutex<PtyManager>`, `Mutex<AppConfig>`, `Mutex<ContextManager>`, `Mutex<Option<LlmClient>>`. All state access is serialized through these mutexes.

## Important Files
- `src-tauri/tauri.conf.json` - Tauri app config (window size, build settings)
- `src-tauri/capabilities/default.json` - Permissions for Tauri plugins
- `src/types/index.ts` - Shared TypeScript interfaces
- `src/styles/main.css` - Global styles (Catppuccin-inspired palette)

## Config Location
- **macOS**: `~/.aiterm/config.json`
- **Linux**: `~/.config/aiterm/config.json`
- **Windows**: `%APPDATA%\aiterm\config.json`

## Dependencies
- **Frontend**: Vue 3.5, Pinia 2.3, xterm.js 5.5 (with fit + web-links addons)
- **Backend**: Tauri 2.0, portable-pty 0.8, tokio 1, reqwest 0.11, keyring 2
