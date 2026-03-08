# aiTerm Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an AI-First local terminal with Ollama and OpenAI-compatible cloud API support.

**Architecture:** Tauri 2.0 desktop app with Vue 3 frontend and Rust backend. Event-driven IPC communication between frontend (xterm.js terminal, AI chat panel) and backend (PTY manager, LLM client, context manager).

**Tech Stack:** Vue 3 + TypeScript + Pinia, xterm.js, Tauri 2.0, Rust, portable-pty, keyring-rs

---

## Phase 1: Project Skeleton (M1)

### Task 1.1: Initialize Tauri 2.0 Project

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`

**Step 1: Create Tauri project**

```bash
cd /Users/luopeng/Documents/GitHub/aiTerm
pnpm create tauri-app@latest . --template vue-ts
```

Select options:
- Package manager: pnpm
- UI template: Vue
- UI flavor: TypeScript

**Step 2: Verify project structure**

```bash
ls -la
ls -la src-tauri/
```

Expected: See `src/`, `src-tauri/`, `package.json`, `vite.config.ts`

**Step 3: Install dependencies**

```bash
pnpm install
```

**Step 4: Verify dev server works**

```bash
pnpm tauri dev
```

Expected: Blank window opens, no errors in terminal

**Step 5: Commit**

```bash
git add .
git commit -m "feat: initialize Tauri 2.0 project with Vue 3 + TypeScript"
```

---

### Task 1.2: Configure Project Dependencies

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add frontend dependencies**

```bash
pnpm add xterm xterm-addon-fit xterm-addon-web-links pinia
pnpm add -D @types/node
```

**Step 2: Update package.json scripts**

Ensure `package.json` has:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

**Step 3: Update Cargo.toml with Rust dependencies**

Replace `src-tauri/Cargo.toml` dependencies section:
```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
futures = "0.3"
portable-pty = "0.8"
keyring = "2"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Step 4: Verify build**

```bash
pnpm tauri dev
```

Expected: App opens without errors

**Step 5: Commit**

```bash
git add package.json pnpm-lock.yaml src-tauri/Cargo.toml
git commit -m "feat: add project dependencies (xterm, pinia, portable-pty, keyring)"
```

---

### Task 1.3: Create Directory Structure

**Files:**
- Create: `src/components/`
- Create: `src/stores/`
- Create: `src/types/`
- Create: `src/styles/`
- Create: `src-tauri/src/pty/`
- Create: `src-tauri/src/llm/`
- Create: `src-tauri/src/config/`
- Create: `src-tauri/src/context/`

**Step 1: Create frontend directories**

```bash
mkdir -p src/components src/stores src/types src/styles
```

**Step 2: Create backend directories**

```bash
mkdir -p src-tauri/src/pty src-tauri/src/llm src-tauri/src/config src-tauri/src/context
```

**Step 3: Create placeholder files**

```bash
touch src/components/Terminal.vue
touch src/components/AIChat.vue
touch src/components/CmdConfirm.vue
touch src/components/Settings.vue
touch src/stores/terminal.ts
touch src/stores/ai.ts
touch src/stores/settings.ts
touch src/types/index.ts
touch src/styles/main.css
touch src-tauri/src/pty/mod.rs
touch src-tauri/src/llm/mod.rs
touch src-tauri/src/config/mod.rs
touch src-tauri/src/context/mod.rs
```

**Step 4: Verify structure**

```bash
tree -L 2 src src-tauri/src
```

**Step 5: Commit**

```bash
git add .
git commit -m "feat: create project directory structure"
```

---

### Task 1.4: Configure Linting and Formatting

**Files:**
- Create: `.eslintrc.cjs`
- Create: `.prettierrc`
- Create: `rustfmt.toml`
- Modify: `package.json`

**Step 1: Install linting tools**

```bash
pnpm add -D eslint @typescript-eslint/eslint-plugin @typescript-eslint/parser eslint-plugin-vue prettier eslint-config-prettier eslint-plugin-prettier
```

**Step 2: Create ESLint config**

Create `.eslintrc.cjs`:
```javascript
module.exports = {
  root: true,
  env: {
    node: true,
    browser: true,
    es2021: true,
  },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended',
    'prettier',
  ],
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    ecmaVersion: 2021,
    sourceType: 'module',
  },
  plugins: ['@typescript-eslint', 'prettier'],
  rules: {
    'prettier/prettier': 'error',
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    '@typescript-eslint/no-explicit-any': 'warn',
    'vue/multi-word-component-names': 'off',
  },
};
```

**Step 3: Create Prettier config**

Create `.prettierrc`:
```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100
}
```

**Step 4: Create rustfmt.toml**

Create `src-tauri/rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
```

**Step 5: Add lint scripts to package.json**

```json
{
  "scripts": {
    "lint": "eslint . --ext .vue,.ts,.tsx --fix",
    "format": "prettier --write \"src/**/*.{ts,vue,css}\"",
    "lint:rust": "cd src-tauri && cargo clippy -- -D warnings"
  }
}
```

**Step 6: Verify linting**

```bash
pnpm lint
cd src-tauri && cargo clippy
```

**Step 7: Commit**

```bash
git add .
git commit -m "feat: configure ESLint, Prettier, and Clippy"
```

---

### Task 1.5: Setup Pinia Store

**Files:**
- Modify: `src/main.ts`
- Create: `src/stores/index.ts`
- Modify: `src/stores/terminal.ts`
- Modify: `src/stores/ai.ts`
- Modify: `src/stores/settings.ts`

**Step 1: Create stores index**

Replace `src/stores/index.ts`:
```typescript
import { createPinia } from 'pinia';

export const pinia = createPinia();

export * from './terminal';
export * from './ai';
export * from './settings';
```

**Step 2: Create terminal store**

Replace `src/stores/terminal.ts`:
```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface TerminalOutput {
  id: string;
  content: string;
  timestamp: number;
  type: 'input' | 'output';
}

export const useTerminalStore = defineStore('terminal', () => {
  const outputs = ref<TerminalOutput[]>([]);
  const isReady = ref(false);

  function addOutput(content: string, type: 'input' | 'output' = 'output') {
    outputs.value.push({
      id: crypto.randomUUID(),
      content,
      timestamp: Date.now(),
      type,
    });
  }

  function clearOutputs() {
    outputs.value = [];
  }

  return {
    outputs,
    isReady,
    addOutput,
    clearOutputs,
  };
});
```

**Step 3: Create AI store**

Replace `src/stores/ai.ts`:
```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

export type LlmProvider = 'ollama' | 'openai';

export const useAiStore = defineStore('ai', () => {
  const messages = ref<ChatMessage[]>([]);
  const provider = ref<LlmProvider>('ollama');
  const model = ref('llama3.2');
  const availableModels = ref<string[]>([]);
  const isStreaming = ref(false);
  const isConnected = ref(false);

  function addMessage(role: ChatMessage['role'], content: string) {
    messages.value.push({
      id: crypto.randomUUID(),
      role,
      content,
      timestamp: Date.now(),
    });
  }

  function clearMessages() {
    messages.value = [];
  }

  return {
    messages,
    provider,
    model,
    availableModels,
    isStreaming,
    isConnected,
    addMessage,
    clearMessages,
  };
});
```

**Step 4: Create settings store**

Replace `src/stores/settings.ts`:
```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface AppSettings {
  llm: {
    provider: 'ollama' | 'openai';
    ollama: {
      host: string;
      model: string;
    };
    openai: {
      apiBase: string;
      model: string;
      hasApiKey: boolean;
    };
  };
  terminal: {
    shell: string;
    fontSize: number;
    fontFamily: string;
    theme: 'dark' | 'light';
  };
  context: {
    maxLines: number;
    maxTokens: number;
  };
}

const defaultSettings: AppSettings = {
  llm: {
    provider: 'ollama',
    ollama: {
      host: 'http://localhost:11434',
      model: 'llama3.2',
    },
    openai: {
      apiBase: 'https://api.openai.com/v1',
      model: 'gpt-4o-mini',
      hasApiKey: false,
    },
  },
  terminal: {
    shell: 'auto',
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, monospace',
    theme: 'dark',
  },
  context: {
    maxLines: 500,
    maxTokens: 4096,
  },
};

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({ ...defaultSettings });
  const isLoading = ref(false);

  async function loadSettings() {
    isLoading.value = true;
    try {
      const loaded = await window.__TAURI_INTERNALS__.invoke<AppSettings>('get_settings');
      if (loaded) {
        settings.value = { ...defaultSettings, ...loaded };
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings() {
    try {
      await window.__TAURI__.invoke('save_settings', { settings: settings.value });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  return {
    settings,
    isLoading,
    loadSettings,
    saveSettings,
  };
});
```

**Step 5: Update main.ts**

Replace `src/main.ts`:
```typescript
import { createApp } from 'vue';
import { pinia } from './stores';
import App from './App.vue';
import './styles/main.css';

const app = createApp(App);
app.use(pinia);
app.mount('#app');
```

**Step 6: Verify build**

```bash
pnpm build
```

Expected: Build succeeds without errors

**Step 7: Commit**

```bash
git add .
git commit -m "feat: setup Pinia stores for terminal, AI, and settings"
```

---

### Task 1.6: Create TypeScript Types

**Files:**
- Modify: `src/types/index.ts`

**Step 1: Define types**

Replace `src/types/index.ts`:
```typescript
// PTY Commands
export interface PtyCreateArgs {
  shell?: string;
}

export interface PtyWriteArgs {
  data: string;
}

export interface PtyResizeArgs {
  cols: number;
  rows: number;
}

// PTY Events
export interface PtyOutputEvent {
  data: string;
}

export interface PtyExitEvent {
  code: number;
}

// AI Commands
export interface AiChatArgs {
  message: string;
  includeContext?: boolean;
}

export interface AiSetProviderArgs {
  provider: 'ollama' | 'openai';
  apiKey?: string;
}

// AI Events
export interface AiStreamEvent {
  content: string;
}

export interface AiErrorEvent {
  message: string;
}

// LLM Response
export interface LlmModel {
  name: string;
  size?: number;
  modified_at?: string;
}

// Tauri command type declarations
declare global {
  interface Window {
    __TAURI__?: {
      invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    };
    __TAURI_INTERNALS__?: {
      invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    };
  }
}
```

**Step 2: Verify TypeScript compilation**

```bash
pnpm exec vue-tsc --noEmit
```

Expected: No type errors

**Step 3: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add TypeScript type definitions for IPC"
```

---

## Phase 2: Terminal Core (M2)

### Task 2.1: Implement PTY Manager Module

**Files:**
- Modify: `src-tauri/src/pty/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create PTY manager**

Replace `src-tauri/src/pty/mod.rs`:
```rust
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Runtime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("PTY creation failed: {0}")]
    CreationFailed(String),
    #[error("PTY write failed: {0}")]
    WriteFailed(String),
    #[error("PTY not initialized")]
    NotInitialized,
    #[error("Reader error: {0}")]
    ReaderError(String),
}

pub struct PtyManager {
    pair: Option<PtyPair>,
    writer: Option<Box<dyn Write + Send>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            pair: None,
            writer: None,
        }
    }

    pub fn create(&mut self, shell: Option<&str>) -> Result<(), PtyError> {
        let pty_system = native_pty_system();

        let shell_cmd = shell.unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "cmd.exe"
            } else {
                "/bin/zsh"
            }
        });

        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreationFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(shell_cmd);
        cmd.env("TERM", "xterm-256color");

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::CreationFailed(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::CreationFailed(e.to_string()))?;

        self.writer = Some(writer);
        self.pair = Some(pair);

        Ok(())
    }

    pub fn write(&mut self, data: &str) -> Result<(), PtyError> {
        let writer = self.writer.as_mut().ok_or(PtyError::NotInitialized)?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| PtyError::WriteFailed(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| PtyError::WriteFailed(e.to_string()))
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let pair = self.pair.as_ref().ok_or(PtyError::NotInitialized)?;
        pair.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::WriteFailed(e.to_string()))
    }

    pub fn start_reader<R: Runtime>(&self, app_handle: AppHandle<R>) -> Result<(), PtyError> {
        let pair = self.pair.as_ref().ok_or(PtyError::NotInitialized)?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::ReaderError(e.to_string()))?;

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        let _ = app_handle.emit("pty_exit", PtyExitPayload { code: 0 });
                        break;
                    }
                    Ok(n) => {
                        if let Ok(data) = String::from_utf8(buffer[..n].to_vec()) {
                            let _ = app_handle.emit("pty_output", PtyOutputPayload { data });
                        }
                    }
                    Err(e) => {
                        tracing::error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

#[derive(Clone, serde::Serialize)]
pub struct PtyOutputPayload {
    pub data: String,
}

#[derive(Clone, serde::Serialize)]
pub struct PtyExitPayload {
    pub code: i32,
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Update lib.rs to export modules**

Replace `src-tauri/src/lib.rs`:
```rust
pub mod pty;
pub mod config;
pub mod llm;
pub mod context;

use pty::{PtyManager, PtyOutputPayload, PtyExitPayload};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub pty: Mutex<PtyManager>,
}

#[tauri::command]
async fn pty_create(
    shell: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut pty = state.pty.lock().map_err(|e| e.to_string())?;

    pty.create(shell.as_deref())
        .map_err(|e| e.to_string())?;

    pty.start_reader(app_handle)
        .map_err(|e| e.to_string())?;

    tracing::info!("PTY created successfully");
    Ok(())
}

#[tauri::command]
fn pty_write(data: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut pty = state.pty.lock().map_err(|e| e.to_string())?;
    pty.write(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn pty_resize(cols: u16, rows: u16, state: State<'_, AppState>) -> Result<(), String> {
    let pty = state.pty.lock().map_err(|e| e.to_string())?;
    pty.resize(cols, rows).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            pty: Mutex::new(PtyManager::new()),
        })
        .invoke_handler(tauri::generate_handler![pty_create, pty_write, pty_resize])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify Rust compilation**

```bash
cd src-tauri && cargo check
```

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: implement PTY manager with create, write, resize commands"
```

---

### Task 2.2: Create Terminal Component

**Files:**
- Modify: `src/components/Terminal.vue`
- Create: `src/components/TerminalXterm.vue`

**Step 1: Install xterm types**

```bash
pnpm add -D @types/xterm
```

**Step 2: Create Terminal component**

Replace `src/components/Terminal.vue`:
```vue
<template>
  <div class="terminal-container" ref="containerRef">
    <div class="terminal" ref="terminalRef"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';
import { useTerminalStore } from '../stores/terminal';
import 'xterm/css/xterm.css';

const containerRef = ref<HTMLElement | null>(null);
const terminalRef = ref<HTMLElement | null>(null);
const terminal = ref<Terminal | null>(null);
const fitAddon = ref<FitAddon | null>(null);

const terminalStore = useTerminalStore();

const props = defineProps<{
  fontSize?: number;
  fontFamily?: string;
  theme?: 'dark' | 'light';
}>();

function initializeTerminal() {
  if (!terminalRef.value) return;

  terminal.value = new Terminal({
    fontSize: props.fontSize || 14,
    fontFamily: props.fontFamily || 'Menlo, Monaco, "Courier New", monospace',
    theme: props.theme === 'light' ? lightTheme : darkTheme,
    cursorBlink: true,
    cursorStyle: 'block',
    allowTransparency: true,
    scrollback: 10000,
  });

  fitAddon.value = new FitAddon();
  terminal.value.loadAddon(fitAddon.value);
  terminal.value.loadAddon(new WebLinksAddon());

  terminal.value.open(terminalRef.value);

  // Fit terminal to container
  setTimeout(() => {
    fitAddon.value?.fit();
  }, 0);

  // Handle user input
  terminal.value.onData((data) => {
    handleInput(data);
  });

  // Handle resize
  terminal.value.onResize(({ cols, rows }) => {
    handleResize(cols, rows);
  });

  terminalStore.isReady = true;
}

async function handleInput(data: string) {
  try {
    await window.__TAURI__?.invoke('pty_write', { data });
    terminalStore.addOutput(data, 'input');
  } catch (error) {
    console.error('Failed to write to PTY:', error);
  }
}

async function handleResize(cols: number, rows: number) {
  try {
    await window.__TAURI__?.invoke('pty_resize', { cols, rows });
  } catch (error) {
    console.error('Failed to resize PTY:', error);
  }
}

function handleWindowResize() {
  fitAddon.value?.fit();
}

function writeToTerminal(data: string) {
  terminal.value?.write(data);
  terminalStore.addOutput(data, 'output');
}

// Listen for PTY output events
function setupEventListeners() {
  // @ts-expect-error - Tauri event listener
  window.__TAURI__?.event?.listen('pty_output', (event: { payload: { data: string } }) => {
    writeToTerminal(event.payload.data);
  });

  // @ts-expect-error - Tauri event listener
  window.__TAURI__?.event?.listen('pty_exit', (event: { payload: { code: number } }) => {
    terminal.value?.write(`\r\n\x1b[33mShell exited with code ${event.payload.code}\x1b[0m\r\n`);
    terminalStore.isReady = false;
  });
}

watch(() => [props.fontSize, props.fontFamily, props.theme], () => {
  if (terminal.value) {
    terminal.value.options.fontSize = props.fontSize;
    terminal.value.options.fontFamily = props.fontFamily;
    terminal.value.options.theme = props.theme === 'light' ? lightTheme : darkTheme;
  }
});

onMounted(async () => {
  initializeTerminal();
  setupEventListeners();
  window.addEventListener('resize', handleWindowResize);

  // Create PTY session
  try {
    await window.__TAURI__?.invoke('pty_create', { shell: null });
    terminal.value?.write('\x1b[32m✓ Terminal ready\x1b[0m\r\n');
  } catch (error) {
    console.error('Failed to create PTY:', error);
    terminal.value?.write('\x1b[31m✗ Failed to create terminal\x1b[0m\r\n');
  }
});

onUnmounted(() => {
  window.removeEventListener('resize', handleWindowResize);
  terminal.value?.dispose();
});

defineExpose({
  write: writeToTerminal,
  fit: () => fitAddon.value?.fit(),
});

const darkTheme = {
  background: '#1e1e2e',
  foreground: '#cdd6f4',
  cursor: '#f5e0dc',
  cursorAccent: '#1e1e2e',
  selectionBackground: '#585b70',
  black: '#45475a',
  red: '#f38ba8',
  green: '#a6e3a1',
  yellow: '#f9e2af',
  blue: '#89b4fa',
  magenta: '#f5c2e7',
  cyan: '#94e2d5',
  white: '#bac2de',
  brightBlack: '#585b70',
  brightRed: '#f38ba8',
  brightGreen: '#a6e3a1',
  brightYellow: '#f9e2af',
  brightBlue: '#89b4fa',
  brightMagenta: '#f5c2e7',
  brightCyan: '#94e2d5',
  brightWhite: '#a6adc8',
};

const lightTheme = {
  background: '#eff1f5',
  foreground: '#4c4f69',
  cursor: '#dc8a78',
  cursorAccent: '#eff1f5',
  selectionBackground: '#acb0be',
  black: '#5c5f77',
  red: '#d20f39',
  green: '#40a02b',
  yellow: '#df8e1d',
  blue: '#1e66f5',
  magenta: '#ea76cb',
  cyan: '#179299',
  white: '#4c4f69',
  brightBlack: '#6c6f85',
  brightRed: '#d20f39',
  brightGreen: '#40a02b',
  brightYellow: '#df8e1d',
  brightBlue: '#1e66f5',
  brightMagenta: '#ea76cb',
  brightCyan: '#179299',
  brightWhite: '#acb0be',
};
</script>

<style scoped>
.terminal-container {
  width: 100%;
  height: 100%;
  padding: 8px;
  box-sizing: border-box;
}

.terminal {
  width: 100%;
  height: 100%;
}
</style>
```

**Step 3: Commit**

```bash
git add src/components/Terminal.vue
git commit -m "feat: create Terminal component with xterm.js integration"
```

---

### Task 2.3: Update App Layout

**Files:**
- Modify: `src/App.vue`
- Modify: `src/styles/main.css`

**Step 1: Update App.vue**

Replace `src/App.vue`:
```vue
<template>
  <div class="app" :class="theme">
    <div class="main-content">
      <Terminal
        :fontSize="settings.settings.terminal.fontSize"
        :fontFamily="settings.settings.terminal.fontFamily"
        :theme="settings.settings.terminal.theme"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import Terminal from './components/Terminal.vue';
import { useSettingsStore } from './stores/settings';

const settings = useSettingsStore();

onMounted(async () => {
  await settings.loadSettings();
});
</script>

<style scoped>
.app {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app.dark {
  background-color: #1e1e2e;
  color: #cdd6f4;
}

.app.light {
  background-color: #eff1f5;
  color: #4c4f69;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}
</style>
```

**Step 2: Update main.css**

Replace `src/styles/main.css`:
```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #585b70;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #6c7086;
}
```

**Step 3: Verify app runs**

```bash
pnpm tauri dev
```

Expected: Terminal window opens, shell is interactive

**Step 4: Commit**

```bash
git add src/App.vue src/styles/main.css
git commit -m "feat: update App layout with terminal as main content"
```

---

## Phase 3: AI Integration (M3)

### Task 3.1: Implement Config Module

**Files:**
- Modify: `src-tauri/src/config/mod.rs`

**Step 1: Create config module**

Replace `src-tauri/src/config/mod.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to get config directory: {0}")]
    DirectoryError(String),
    #[error("Failed to read config: {0}")]
    ReadError(String),
    #[error("Failed to write config: {0}")]
    WriteError(String),
    #[error("Failed to parse config: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_base: String,
    pub model: String,
    pub has_api_key: bool,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_base: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            has_api_key: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub ollama: OllamaConfig,
    pub openai: OpenAiConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            ollama: OllamaConfig::default(),
            openai: OpenAiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub shell: String,
    pub font_size: u8,
    pub font_family: String,
    pub theme: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: "auto".to_string(),
            font_size: 14,
            font_family: "Menlo, Monaco, monospace".to_string(),
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_lines: usize,
    pub max_tokens: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_lines: 500,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub terminal: TerminalConfig,
    pub context: ContextConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            terminal: TerminalConfig::default(),
            context: ContextConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or_else(|| {
            ConfigError::DirectoryError("Cannot determine home directory".to_string())
        })?;
        Ok(home.join(".aiterm"))
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        tracing::info!("Config loaded from {:?}", path);
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        tracing::info!("Config saved to {:?}", path);
        Ok(())
    }
}
```

**Step 2: Add dirs dependency to Cargo.toml**

```bash
cd src-tauri && cargo add dirs
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/config/mod.rs src-tauri/Cargo.toml
git commit -m "feat: implement config module with JSON persistence"
```

---

### Task 3.2: Implement Context Manager

**Files:**
- Modify: `src-tauri/src/context/mod.rs`

**Step 1: Create context manager**

Replace `src-tauri/src/context/mod.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEntry {
    pub content: String,
    pub entry_type: EntryType,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    Input,
    Output,
}

pub struct ContextManager {
    buffer: VecDeque<TerminalEntry>,
    max_lines: usize,
}

impl ContextManager {
    pub fn new(max_lines: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_lines),
            max_lines,
        }
    }

    pub fn push(&mut self, content: String, entry_type: EntryType) {
        let entry = TerminalEntry {
            content,
            entry_type,
            timestamp: chrono::Utc::now().timestamp(),
        };

        if self.buffer.len() >= self.max_lines {
            self.buffer.pop_front();
        }

        self.buffer.push_back(entry);
    }

    pub fn push_input(&mut self, content: String) {
        self.push(content, EntryType::Input);
    }

    pub fn push_output(&mut self, content: String) {
        self.push(content, EntryType::Output);
    }

    pub fn build_context(&self, max_tokens: usize) -> String {
        let mut context = String::new();
        let mut estimated_tokens = 0;

        // Iterate in reverse to get most recent entries first
        for entry in self.buffer.iter().rev() {
            let entry_text = match entry.entry_type {
                EntryType::Input => format!("$ {}\n", entry.content.trim()),
                EntryType::Output => format!("{}\n", entry.content.trim()),
            };

            // Rough estimate: 1 token ≈ 4 characters
            let entry_tokens = entry_text.len() / 4;

            if estimated_tokens + entry_tokens > max_tokens {
                break;
            }

            context = entry_text + &context;
            estimated_tokens += entry_tokens;
        }

        context
    }

    pub fn get_recent_errors(&self, count: usize) -> Vec<&TerminalEntry> {
        self.buffer
            .iter()
            .rev()
            .filter(|e| {
                matches!(e.entry_type, EntryType::Output)
                    && (e.content.contains("error")
                        || e.content.contains("Error")
                        || e.content.contains("ERROR")
                        || e.content.contains("failed")
                        || e.content.contains("Failed"))
            })
            .take(count)
            .collect()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(500)
    }
}
```

**Step 2: Add chrono dependency**

```bash
cd src-tauri && cargo add chrono --features serde
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/context/mod.rs src-tauri/Cargo.toml
git commit -m "feat: implement context manager with ring buffer"
```

---

### Task 3.3: Implement LLM Client

**Files:**
- Modify: `src-tauri/src/llm/mod.rs`

**Step 1: Create LLM client**

Replace `src-tauri/src/llm/mod.rs`:
```rust
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("No API key configured")]
    NoApiKey,
}

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Ollama { host: String, model: String },
    OpenAi { api_base: String, model: String, api_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaResponse {
    message: Option<ChatMessage>,
    done: bool,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChoice {
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
    role: Option<String>,
}

pub struct LlmClient {
    client: Client,
    provider: LlmProvider,
}

impl LlmClient {
    pub fn new(provider: LlmProvider) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, provider }
    }

    pub fn set_provider(&mut self, provider: LlmProvider) {
        self.provider = provider;
    }

    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, .. } => {
                let url = format!("{}/api/tags", host);
                let response = self.client.get(&url).send().await?;
                let json: serde_json::Value = response.json().await?;

                let models = json["models"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m["name"].as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(models)
            }
            LlmProvider::OpenAi { api_base, api_key, .. } => {
                let url = format!("{}/models", api_base);
                let response = self
                    .client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send()
                    .await?;

                let json: serde_json::Value = response.json().await?;

                let models = json["data"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m["id"].as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(models)
            }
        }
    }

    pub async fn test_connection(&self) -> Result<bool, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, .. } => {
                let url = format!("{}/api/version", host);
                let response = self.client.get(&url).send().await?;
                Ok(response.status().is_success())
            }
            LlmProvider::OpenAi { api_base, api_key, .. } => {
                let url = format!("{}/models", api_base);
                let response = self
                    .client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send()
                    .await?;
                Ok(response.status().is_success())
            }
        }
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<impl futures::Stream<Item = Result<String, LlmError>>, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, model } => {
                self.ollama_chat_stream(host, model, messages).await
            }
            LlmProvider::OpenAi { api_base, model, api_key } => {
                self.openai_chat_stream(api_base, model, api_key, messages).await
            }
        }
    }

    async fn ollama_chat_stream(
        &self,
        host: &str,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<impl futures::Stream<Item = Result<String, LlmError>>, LlmError> {
        let url = format!("{}/api/chat", host);
        let request = OllamaRequest {
            model: model.to_string(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let stream = response.bytes_stream().map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    // Each line is a JSON object
                    for line in text.lines() {
                        if line.is_empty() {
                            continue;
                        }
                        if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                            if let Some(message) = response.message {
                                return Ok(message.content);
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(LlmError::HttpError(e)),
            }
        });

        Ok(stream)
    }

    async fn openai_chat_stream(
        &self,
        api_base: &str,
        model: &str,
        api_key: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<impl futures::Stream<Item = Result<String, LlmError>>, LlmError> {
        let url = format!("{}/chat/completions", api_base);
        let request = OpenAiRequest {
            model: model.to_string(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let stream = response.bytes_stream().map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if !line.starts_with("data: ") {
                            continue;
                        }
                        let data = &line[6..];
                        if data == "[DONE]" {
                            return Ok(String::new());
                        }
                        if let Ok(response) = serde_json::from_str::<OpenAiResponse>(data) {
                            if let Some(choice) = response.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    return Ok(content.clone());
                                }
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(LlmError::HttpError(e)),
            }
        });

        Ok(stream)
    }
}

pub fn build_system_prompt() -> String {
    r#"你是运维助手，根据终端历史帮助用户解决问题。

规则：
1. 只给出可直接执行的命令，用 ```bash 包裹
2. 简短解释原因（1-2 句）
3. 不确定时询问更多信息
4. 危险操作（rm、sudo 等）需特别提醒
5. 优先使用用户已使用的工具

输出格式：
命令：
```bash
<命令>
```

原因：<简短解释>"#
        .to_string()
}
```

**Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 3: Commit**

```bash
git add src-tauri/src/llm/mod.rs
git commit -m "feat: implement LLM client with Ollama and OpenAI support"
```

---

### Task 3.4: Integrate LLM Commands into Tauri

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/llm/mod.rs`

**Step 1: Update lib.rs with LLM commands**

Add to `src-tauri/src/lib.rs`:
```rust
use llm::{LlmClient, LlmProvider, ChatMessage, build_system_prompt};
use context::ContextManager;
use config::AppConfig;
use keyring::Entry;

pub struct AppState {
    pub pty: Mutex<PtyManager>,
    pub config: Mutex<AppConfig>,
    pub context: Mutex<ContextManager>,
    pub llm: Mutex<Option<LlmClient>>,
}

fn get_api_key() -> Result<String, String> {
    let entry = Entry::new("aiTerm", "openai_api_key")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;
    entry.get_password().map_err(|e| format!("Failed to get API key: {}", e))
}

fn set_api_key(key: &str) -> Result<(), String> {
    let entry = Entry::new("aiTerm", "openai_api_key")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;
    entry.set_password(key).map_err(|e| format!("Failed to save API key: {}", e))
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
async fn save_settings(settings: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = settings.clone();
    config.save().map_err(|e| e.to_string())?;

    // Update LLM client with new settings
    let provider = match settings.llm.provider.as_str() {
        "openai" => {
            let api_key = get_api_key()?;
            LlmProvider::OpenAi {
                api_base: settings.llm.openai.api_base,
                model: settings.llm.openai.model,
                api_key,
            }
        }
        _ => LlmProvider::Ollama {
            host: settings.llm.ollama.host,
            model: settings.llm.ollama.model,
        },
    };

    let mut llm = state.llm.lock().map_err(|e| e.to_string())?;
    *llm = Some(LlmClient::new(provider));

    Ok(())
}

#[tauri::command]
async fn ai_set_provider(
    provider: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let llm_provider = match provider.as_str() {
        "openai" => {
            let key = api_key.ok_or("API key required for OpenAI provider")?;
            set_api_key(&key)?;
            LlmProvider::OpenAi {
                api_base: config.llm.openai.api_base.clone(),
                model: config.llm.openai.model.clone(),
                api_key: key,
            }
        }
        _ => LlmProvider::Ollama {
            host: config.llm.ollama.host.clone(),
            model: config.llm.ollama.model.clone(),
        },
    };

    let mut llm = state.llm.lock().map_err(|e| e.to_string())?;
    *llm = Some(LlmClient::new(llm_provider));

    Ok(())
}

#[tauri::command]
async fn ai_list_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let llm = state.llm.lock().map_err(|e| e.to_string())?;
    let client = llm.as_ref().ok_or("LLM client not initialized")?;
    client.list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_test_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let llm = state.llm.lock().map_err(|e| e.to_string())?;
    let client = llm.as_ref().ok_or("LLM client not initialized")?;
    client.test_connection().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_chat(
    message: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let context = {
        let ctx = state.context.lock().map_err(|e| e.to_string())?;
        let config = state.config.lock().map_err(|e| e.to_string())?;
        ctx.build_context(config.context.max_tokens)
    };

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: build_system_prompt(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("[终端历史]\n{}\n\n[用户问题]\n{}", context, message),
        },
    ];

    let llm = state.llm.lock().map_err(|e| e.to_string())?;
    let client = llm.as_ref().ok_or("LLM client not initialized")?.clone();

    // Clone for async task
    let handle = app_handle.clone();

    tokio::spawn(async move {
        match client.chat_stream(messages).await {
            Ok(mut stream) => {
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(content) if !content.is_empty() => {
                            let _ = handle.emit("ai_stream", content);
                        }
                        Err(e) => {
                            let _ = handle.emit("ai_error", e.to_string());
                            break;
                        }
                        _ => {}
                    }
                }
                let _ = handle.emit("ai_done", ());
            }
            Err(e) => {
                let _ = handle.emit("ai_error", e.to_string());
            }
        }
    });

    Ok(())
}
```

**Step 2: Update AppState initialization and invoke_handler**

```rust
.manage(AppState {
    pty: Mutex::new(PtyManager::new()),
    config: Mutex::new(AppConfig::load().unwrap_or_default()),
    context: Mutex::new(ContextManager::new(500)),
    llm: Mutex::new(None),
})
.invoke_handler(tauri::generate_handler![
    pty_create, pty_write, pty_resize,
    get_settings, save_settings,
    ai_set_provider, ai_list_models, ai_test_connection, ai_chat
])
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: integrate LLM commands with Tauri IPC"
```

---

## Phase 4: AI UI (M4)

### Task 4.1: Create AI Chat Panel Component

**Files:**
- Modify: `src/components/AIChat.vue`

**Step 1: Create AI Chat component**

Replace `src/components/AIChat.vue`:
```vue
<template>
  <div class="ai-chat" :class="{ collapsed: isCollapsed }">
    <div class="chat-header">
      <span class="title">AI Assistant</span>
      <div class="header-actions">
        <span class="provider-badge" :class="aiStore.provider">
          {{ aiStore.provider }}
        </span>
        <button class="btn-icon" @click="toggleCollapse">
          {{ isCollapsed ? '◀' : '▶' }}
        </button>
      </div>
    </div>

    <div v-if="!isCollapsed" class="chat-content">
      <div class="messages" ref="messagesRef">
        <div
          v-for="msg in aiStore.messages"
          :key="msg.id"
          class="message"
          :class="msg.role"
        >
          <div class="message-header">
            <span class="role">{{ msg.role }}</span>
            <span class="time">{{ formatTime(msg.timestamp) }}</span>
          </div>
          <div class="message-content" v-html="renderMarkdown(msg.content)"></div>
          <div v-if="msg.role === 'assistant'" class="message-actions">
            <button
              v-for="(cmd, i) in extractCommands(msg.content)"
              :key="i"
              class="cmd-btn"
              @click="executeCommand(cmd)"
            >
              ▶ {{ cmd }}
            </button>
          </div>
        </div>
        <div v-if="aiStore.isStreaming" class="message assistant streaming">
          <div class="message-header">
            <span class="role">assistant</span>
            <span class="streaming-indicator">●●●</span>
          </div>
          <div class="message-content">{{ streamingContent }}</div>
        </div>
      </div>

      <div class="input-area">
        <textarea
          v-model="userInput"
          placeholder="Ask AI for help..."
          @keydown.enter.exact.prevent="sendMessage"
          :disabled="aiStore.isStreaming"
        ></textarea>
        <button
          class="send-btn"
          @click="sendMessage"
          :disabled="!userInput.trim() || aiStore.isStreaming"
        >
          Send
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useAiStore } from '../stores/ai';

const aiStore = useAiStore();
const userInput = ref('');
const streamingContent = ref('');
const messagesRef = ref<HTMLElement | null>(null);
const isCollapsed = ref(false);

const emit = defineEmits<{
  executeCommand: [command: string];
}>();

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value;
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function renderMarkdown(content: string): string {
  // Simple markdown rendering for code blocks
  return content
    .replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre><code class="language-$1">$2</code></pre>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\n/g, '<br>');
}

function extractCommands(content: string): string[] {
  const regex = /```(?:bash|sh|shell)?\n([\s\S]*?)```/g;
  const commands: string[] = [];
  let match;
  while ((match = regex.exec(content)) !== null) {
    const cmd = match[1].trim();
    if (cmd && !cmd.includes('\n')) {
      commands.push(cmd);
    }
  }
  return commands;
}

async function sendMessage() {
  if (!userInput.value.trim() || aiStore.isStreaming) return;

  const message = userInput.value.trim();
  userInput.value = '';
  aiStore.addMessage('user', message);
  aiStore.isStreaming = true;
  streamingContent.value = '';

  try {
    await window.__TAURI__?.invoke('ai_chat', { message, includeContext: true });
  } catch (error) {
    console.error('Failed to send message:', error);
    aiStore.isStreaming = false;
  }

  await nextTick();
  scrollToBottom();
}

function executeCommand(command: string) {
  emit('executeCommand', command);
}

function scrollToBottom() {
  if (messagesRef.value) {
    messagesRef.value.scrollTop = messagesRef.value.scrollHeight;
  }
}

function setupEventListeners() {
  // @ts-expect-error - Tauri event
  window.__TAURI__?.event?.listen('ai_stream', (event: { payload: string }) => {
    streamingContent.value += event.payload;
    scrollToBottom();
  });

  // @ts-expect-error - Tauri event
  window.__TAURI__?.event?.listen('ai_done', () => {
    if (streamingContent.value) {
      aiStore.addMessage('assistant', streamingContent.value);
    }
    streamingContent.value = '';
    aiStore.isStreaming = false;
  });

  // @ts-expect-error - Tauri event
  window.__TAURI__?.event?.listen('ai_error', (event: { payload: string }) => {
    aiStore.addMessage('assistant', `Error: ${event.payload}`);
    streamingContent.value = '';
    aiStore.isStreaming = false;
  });
}

watch(() => aiStore.messages.length, () => {
  nextTick(scrollToBottom);
});

onMounted(() => {
  setupEventListeners();
});

onUnmounted(() => {
  // Cleanup listeners if needed
});
</script>

<style scoped>
.ai-chat {
  width: 350px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1e1e2e;
  border-left: 1px solid #313244;
}

.ai-chat.collapsed {
  width: 40px;
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #313244;
  background: #181825;
}

.title {
  font-weight: 600;
  color: #cdd6f4;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.provider-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
}

.provider-badge.ollama {
  background: #a6e3a1;
  color: #1e1e2e;
}

.provider-badge.openai {
  background: #89b4fa;
  color: #1e1e2e;
}

.btn-icon {
  background: none;
  border: none;
  color: #6c7086;
  cursor: pointer;
  padding: 4px;
}

.btn-icon:hover {
  color: #cdd6f4;
}

.chat-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.message {
  margin-bottom: 16px;
  padding: 12px;
  border-radius: 8px;
}

.message.user {
  background: #313244;
}

.message.assistant {
  background: #181825;
  border: 1px solid #313244;
}

.message-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
}

.role {
  text-transform: uppercase;
  font-weight: 600;
  color: #89b4fa;
}

.time {
  color: #6c7086;
}

.message-content {
  color: #cdd6f4;
  line-height: 1.5;
}

.message-content :deep(pre) {
  background: #11111b;
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 8px 0;
}

.message-content :deep(code) {
  font-family: 'Menlo', monospace;
  font-size: 13px;
}

.message-actions {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.cmd-btn {
  background: #313244;
  border: 1px solid #45475a;
  color: #a6e3a1;
  padding: 6px 12px;
  border-radius: 4px;
  font-family: 'Menlo', monospace;
  font-size: 12px;
  cursor: pointer;
}

.cmd-btn:hover {
  background: #45475a;
}

.streaming-indicator {
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 1; }
}

.input-area {
  padding: 12px;
  border-top: 1px solid #313244;
  display: flex;
  gap: 8px;
}

.input-area textarea {
  flex: 1;
  background: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  padding: 8px 12px;
  color: #cdd6f4;
  font-family: inherit;
  font-size: 14px;
  resize: none;
  height: 60px;
}

.input-area textarea:focus {
  outline: none;
  border-color: #89b4fa;
}

.send-btn {
  background: #89b4fa;
  color: #1e1e2e;
  border: none;
  border-radius: 6px;
  padding: 8px 16px;
  font-weight: 600;
  cursor: pointer;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-btn:hover:not(:disabled) {
  background: #b4befe;
}
</style>
```

**Step 2: Commit**

```bash
git add src/components/AIChat.vue
git commit -m "feat: create AI chat panel component with streaming support"
```

---

### Task 4.2: Create Command Confirmation Modal

**Files:**
- Modify: `src/components/CmdConfirm.vue`

**Step 1: Create confirmation modal**

Replace `src/components/CmdConfirm.vue`:
```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click.self="cancel">
      <div class="modal">
        <div class="modal-header">
          <h3>AI 建议执行以下命令</h3>
        </div>
        <div class="modal-body">
          <div class="command-box">
            <code>{{ command }}</code>
          </div>
          <p v-if="reason" class="reason">{{ reason }}</p>
          <p v-if="isDangerous" class="warning">
            ⚠️ 此命令可能具有破坏性，请确认后执行
          </p>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="cancel">
            取消 (n)
          </button>
          <button class="btn btn-warning" @click="edit">
            编辑 (e)
          </button>
          <button class="btn btn-primary" @click="execute">
            执行 (y)
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';

const props = defineProps<{
  visible: boolean;
  command: string;
  reason?: string;
}>();

const emit = defineEmits<{
  execute: [];
  edit: [];
  cancel: [];
}>();

const isDangerous = computed(() => {
  const dangerousPatterns = [
    /\brm\s/,
    /\brm\b/,
    /sudo/,
    /chmod/,
    /chown/,
    /mkfs/,
    /dd\s/,
    />\s*\//,
    /\|\s*sh/,
    /curl.*\|/,
    /wget.*\|/,
  ];
  return dangerousPatterns.some((p) => p.test(props.command));
});

function execute() {
  emit('execute');
}

function edit() {
  emit('edit');
}

function cancel() {
  emit('cancel');
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;

  switch (e.key.toLowerCase()) {
    case 'y':
    case 'enter':
      execute();
      break;
    case 'n':
    case 'escape':
      cancel();
      break;
    case 'e':
      edit();
      break;
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: #1e1e2e;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
  border: 1px solid #313244;
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid #313244;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  color: #cdd6f4;
}

.modal-body {
  padding: 20px;
}

.command-box {
  background: #11111b;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
}

.command-box code {
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 14px;
  color: #a6e3a1;
  word-break: break-all;
}

.reason {
  color: #a6adc8;
  font-size: 14px;
  margin: 0;
}

.warning {
  color: #f9e2af;
  font-size: 13px;
  margin-top: 12px;
  padding: 8px 12px;
  background: rgba(249, 226, 175, 0.1);
  border-radius: 6px;
}

.modal-footer {
  padding: 16px 20px;
  border-top: 1px solid #313244;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background 0.2s;
}

.btn-primary {
  background: #a6e3a1;
  color: #1e1e2e;
}

.btn-primary:hover {
  background: #94e2d5;
}

.btn-secondary {
  background: #45475a;
  color: #cdd6f4;
}

.btn-secondary:hover {
  background: #585b70;
}

.btn-warning {
  background: #f9e2af;
  color: #1e1e2e;
}

.btn-warning:hover {
  background: #f5c2e7;
}
</style>
```

**Step 2: Commit**

```bash
git add src/components/CmdConfirm.vue
git commit -m "feat: create command confirmation modal with danger detection"
```

---

### Task 4.3: Create Settings Component

**Files:**
- Modify: `src/components/Settings.vue`

**Step 1: Create settings component**

Replace `src/components/Settings.vue`:
```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="settings-overlay" @click.self="close">
      <div class="settings-panel">
        <div class="settings-header">
          <h2>Settings</h2>
          <button class="close-btn" @click="close">×</button>
        </div>

        <div class="settings-content">
          <!-- LLM Settings -->
          <section class="settings-section">
            <h3>AI Provider</h3>

            <div class="form-group">
              <label>Provider</label>
              <select v-model="localSettings.llm.provider" @change="onProviderChange">
                <option value="ollama">Ollama (Local)</option>
                <option value="openai">OpenAI Compatible (Cloud)</option>
              </select>
            </div>

            <!-- Ollama Settings -->
            <template v-if="localSettings.llm.provider === 'ollama'">
              <div class="form-group">
                <label>Ollama Host</label>
                <input
                  v-model="localSettings.llm.ollama.host"
                  type="text"
                  placeholder="http://localhost:11434"
                />
              </div>
              <div class="form-group">
                <label>Model</label>
                <select v-model="localSettings.llm.ollama.model">
                  <option v-for="model in ollamaModels" :key="model" :value="model">
                    {{ model }}
                  </option>
                </select>
                <button class="btn-small" @click="refreshOllamaModels">Refresh</button>
              </div>
              <div class="form-group">
                <button class="btn" @click="testOllamaConnection">
                  {{ testingConnection ? 'Testing...' : 'Test Connection' }}
                </button>
                <span v-if="connectionStatus" :class="connectionStatus">
                  {{ connectionStatus }}
                </span>
              </div>
            </template>

            <!-- OpenAI Settings -->
            <template v-else>
              <div class="form-group">
                <label>API Base URL</label>
                <input
                  v-model="localSettings.llm.openai.api_base"
                  type="text"
                  placeholder="https://api.openai.com/v1"
                />
              </div>
              <div class="form-group">
                <label>API Key</label>
                <div class="api-key-input">
                  <input
                    v-model="apiKeyInput"
                    :type="showApiKey ? 'text' : 'password'"
                    placeholder="sk-..."
                  />
                  <button class="btn-icon" @click="showApiKey = !showApiKey">
                    {{ showApiKey ? '🙈' : '👁️' }}
                  </button>
                </div>
              </div>
              <div class="form-group">
                <label>Model</label>
                <select v-model="localSettings.llm.openai.model">
                  <option value="gpt-4o-mini">GPT-4o Mini</option>
                  <option value="gpt-4o">GPT-4o</option>
                  <option value="gpt-4-turbo">GPT-4 Turbo</option>
                  <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                </select>
              </div>
            </template>
          </section>

          <!-- Terminal Settings -->
          <section class="settings-section">
            <h3>Terminal</h3>

            <div class="form-group">
              <label>Shell</label>
              <select v-model="localSettings.terminal.shell">
                <option value="auto">Auto Detect</option>
                <option value="/bin/zsh">/bin/zsh</option>
                <option value="/bin/bash">/bin/bash</option>
                <option value="/bin/fish">/bin/fish</option>
              </select>
            </div>

            <div class="form-group">
              <label>Font Size</label>
              <input
                v-model.number="localSettings.terminal.font_size"
                type="number"
                min="10"
                max="24"
              />
            </div>

            <div class="form-group">
              <label>Font Family</label>
              <input
                v-model="localSettings.terminal.font_family"
                type="text"
              />
            </div>

            <div class="form-group">
              <label>Theme</label>
              <select v-model="localSettings.terminal.theme">
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>
          </section>

          <!-- Context Settings -->
          <section class="settings-section">
            <h3>Context</h3>

            <div class="form-group">
              <label>Max History Lines</label>
              <input
                v-model.number="localSettings.context.max_lines"
                type="number"
                min="100"
                max="2000"
              />
            </div>

            <div class="form-group">
              <label>Max Context Tokens</label>
              <input
                v-model.number="localSettings.context.max_tokens"
                type="number"
                min="1024"
                max="8192"
              />
            </div>
          </section>
        </div>

        <div class="settings-footer">
          <button class="btn btn-secondary" @click="resetToDefaults">Reset</button>
          <button class="btn btn-primary" @click="save">Save</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useSettingsStore, type AppSettings } from '../stores/settings';
import { useAiStore } from '../stores/ai';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const settingsStore = useSettingsStore();
const aiStore = useAiStore();

const localSettings = ref<AppSettings>(JSON.parse(JSON.stringify(settingsStore.settings)));
const apiKeyInput = ref('');
const showApiKey = ref(false);
const ollamaModels = ref<string[]>([]);
const testingConnection = ref(false);
const connectionStatus = ref<'success' | 'error' | ''>('');

function close() {
  emit('close');
}

async function onProviderChange() {
  if (localSettings.value.llm.provider === 'ollama') {
    await refreshOllamaModels();
  }
}

async function refreshOllamaModels() {
  try {
    const models = await window.__TAURI__?.invoke<string[]>('ai_list_models');
    if (models) {
      ollamaModels.value = models;
    }
  } catch (error) {
    console.error('Failed to fetch Ollama models:', error);
  }
}

async function testOllamaConnection() {
  testingConnection.value = true;
  connectionStatus.value = '';

  try {
    const success = await window.__TAURI__?.invoke<boolean>('ai_test_connection');
    connectionStatus.value = success ? 'success' : 'error';
  } catch {
    connectionStatus.value = 'error';
  } finally {
    testingConnection.value = false;
  }
}

function resetToDefaults() {
  // Reset to default values
  localSettings.value = {
    llm: {
      provider: 'ollama',
      ollama: { host: 'http://localhost:11434', model: 'llama3.2' },
      openai: { api_base: 'https://api.openai.com/v1', model: 'gpt-4o-mini', has_api_key: false },
    },
    terminal: {
      shell: 'auto',
      font_size: 14,
      font_family: 'Menlo, Monaco, monospace',
      theme: 'dark',
    },
    context: { max_lines: 500, max_tokens: 4096 },
  };
}

async function save() {
  settingsStore.settings = localSettings.value;

  // Save API key separately if provided
  if (apiKeyInput.value && localSettings.value.llm.provider === 'openai') {
    try {
      await window.__TAURI__?.invoke('ai_set_provider', {
        provider: 'openai',
        apiKey: apiKeyInput.value,
      });
      localSettings.value.llm.openai.has_api_key = true;
    } catch (error) {
      console.error('Failed to save API key:', error);
    }
  }

  await settingsStore.saveSettings();
  close();
}

watch(
  () => props.visible,
  async (visible) => {
    if (visible) {
      localSettings.value = JSON.parse(JSON.stringify(settingsStore.settings));
      connectionStatus.value = '';

      if (localSettings.value.llm.provider === 'ollama') {
        await refreshOllamaModels();
      }
    }
  }
);

onMounted(async () => {
  if (localSettings.value.llm.provider === 'ollama') {
    await refreshOllamaModels();
  }
});
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-panel {
  background: #1e1e2e;
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  border: 1px solid #313244;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #313244;
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
  color: #cdd6f4;
}

.close-btn {
  background: none;
  border: none;
  color: #6c7086;
  font-size: 24px;
  cursor: pointer;
}

.close-btn:hover {
  color: #cdd6f4;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section h3 {
  margin: 0 0 16px;
  font-size: 14px;
  text-transform: uppercase;
  color: #89b4fa;
  letter-spacing: 0.5px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #a6adc8;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 10px 12px;
  background: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  color: #cdd6f4;
  font-size: 14px;
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: #89b4fa;
}

.api-key-input {
  display: flex;
  gap: 8px;
}

.api-key-input input {
  flex: 1;
}

.btn-icon {
  background: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
}

.btn {
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.btn-primary {
  background: #89b4fa;
  color: #1e1e2e;
}

.btn-secondary {
  background: #45475a;
  color: #cdd6f4;
}

.btn-small {
  padding: 6px 12px;
  font-size: 12px;
  background: #45475a;
  color: #cdd6f4;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 8px;
}

.success {
  color: #a6e3a1;
  margin-left: 12px;
}

.error {
  color: #f38ba8;
  margin-left: 12px;
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid #313244;
}
</style>
```

**Step 2: Commit**

```bash
git add src/components/Settings.vue
git commit -m "feat: create settings component with LLM provider configuration"
```

---

### Task 4.4: Integrate Components into App

**Files:**
- Modify: `src/App.vue`

**Step 1: Update App.vue with all components**

Replace `src/App.vue`:
```vue
<template>
  <div class="app" :class="settings.settings.terminal.theme">
    <div class="toolbar">
      <button class="toolbar-btn" @click="showSettings = true" title="Settings">
        ⚙️
      </button>
      <button class="toolbar-btn" @click="toggleAI" :class="{ active: showAI }" title="Toggle AI">
        🤖
      </button>
    </div>

    <div class="main-content">
      <Terminal
        ref="terminalRef"
        :fontSize="settings.settings.terminal.font_size"
        :fontFamily="settings.settings.terminal.font_family"
        :theme="settings.settings.terminal.theme"
        @commandDetected="onCommandDetected"
      />

      <AIChat
        v-if="showAI"
        @executeCommand="onExecuteCommand"
      />
    </div>

    <CmdConfirm
      :visible="showConfirm"
      :command="pendingCommand"
      :reason="pendingReason"
      @execute="confirmExecute"
      @edit="editCommand"
      @cancel="cancelCommand"
    />

    <Settings
      :visible="showSettings"
      @close="showSettings = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Terminal from './components/Terminal.vue';
import AIChat from './components/AIChat.vue';
import CmdConfirm from './components/CmdConfirm.vue';
import Settings from './components/Settings.vue';
import { useSettingsStore } from './stores/settings';
import { useAiStore } from './stores/ai';

const settings = useSettingsStore();
const aiStore = useAiStore();

const terminalRef = ref<InstanceType<typeof Terminal> | null>(null);
const showAI = ref(true);
const showSettings = ref(false);
const showConfirm = ref(false);
const pendingCommand = ref('');
const pendingReason = ref('');

function toggleAI() {
  showAI.value = !showAI.value;
}

function onCommandDetected(command: string, reason: string) {
  pendingCommand.value = command;
  pendingReason.value = reason;
  showConfirm.value = true;
}

function onExecuteCommand(command: string) {
  pendingCommand.value = command;
  pendingReason.value = 'Requested from AI chat';
  showConfirm.value = true;
}

async function confirmExecute() {
  if (terminalRef.value && pendingCommand.value) {
    await window.__TAURI__?.invoke('pty_write', { data: pendingCommand.value + '\n' });
  }
  showConfirm.value = false;
  pendingCommand.value = '';
  pendingReason.value = '';
}

function editCommand() {
  // For now, just close - in future could open inline editor
  showConfirm.value = false;
}

function cancelCommand() {
  showConfirm.value = false;
  pendingCommand.value = '';
  pendingReason.value = '';
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Space to focus AI input
  if (e.ctrlKey && e.code === 'Space') {
    e.preventDefault();
    showAI.value = true;
  }
  // Ctrl+, for settings
  if (e.ctrlKey && e.key === ',') {
    e.preventDefault();
    showSettings.value = !showSettings.value;
  }
}

onMounted(async () => {
  await settings.loadSettings();
  window.addEventListener('keydown', handleKeydown);

  // Initialize AI client
  try {
    await window.__TAURI__?.invoke('ai_set_provider', {
      provider: settings.settings.llm.provider,
      apiKey: null,
    });
  } catch (error) {
    console.error('Failed to initialize AI client:', error);
  }
});
</script>

<style scoped>
.app {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app.dark {
  background-color: #1e1e2e;
  color: #cdd6f4;
}

.app.light {
  background-color: #eff1f5;
  color: #4c4f69;
}

.toolbar {
  display: flex;
  gap: 4px;
  padding: 4px 8px;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.toolbar-btn {
  background: none;
  border: none;
  padding: 6px 10px;
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
}

.toolbar-btn:hover {
  background: #313244;
}

.toolbar-btn.active {
  background: #45475a;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}
</style>
```

**Step 2: Verify app runs**

```bash
pnpm tauri dev
```

Expected: Full app with terminal, AI panel, and toolbar

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: integrate all components with keyboard shortcuts"
```

---

## Phase 5: Infrastructure (M5)

### Task 5.1: Update PTY to Capture Context

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add context capture to PTY output**

Update the PTY output event handler in `lib.rs` to also capture context:

```rust
// In the pty_create command, after creating the PTY:
// Add context capture when output is received
```

**Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add context capture to PTY output"
```

---

### Task 5.2: Add Error Handling UI

**Files:**
- Create: `src/components/Toast.vue`
- Modify: `src/App.vue`

**Step 1: Create Toast component**

Create `src/components/Toast.vue`:
```vue
<template>
  <Teleport to="body">
    <div class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="toast.type"
      >
        <span class="toast-icon">{{ toast.type === 'error' ? '✗' : '✓' }}</span>
        <span class="toast-message">{{ toast.message }}</span>
        <button class="toast-close" @click="removeToast(toast.id)">×</button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info';
}

const toasts = ref<Toast[]>([]);

function addToast(message: string, type: Toast['type'] = 'info') {
  const id = crypto.randomUUID();
  toasts.value.push({ id, message, type });

  setTimeout(() => {
    removeToast(id);
  }, 5000);
}

function removeToast(id: string) {
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index > -1) {
    toasts.value.splice(index, 1);
  }
}

defineExpose({ addToast });
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 2000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #1e1e2e;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  animation: slideIn 0.3s ease;
}

.toast.error {
  border-left: 3px solid #f38ba8;
}

.toast.success {
  border-left: 3px solid #a6e3a1;
}

.toast.info {
  border-left: 3px solid #89b4fa;
}

.toast-icon {
  font-size: 14px;
}

.toast.error .toast-icon {
  color: #f38ba8;
}

.toast.success .toast-icon {
  color: #a6e3a1;
}

.toast.info .toast-icon {
  color: #89b4fa;
}

.toast-message {
  flex: 1;
  color: #cdd6f4;
  font-size: 14px;
}

.toast-close {
  background: none;
  border: none;
  color: #6c7086;
  font-size: 18px;
  cursor: pointer;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
```

**Step 2: Commit**

```bash
git add src/components/Toast.vue
git commit -m "feat: add toast notification component"
```

---

### Task 5.3: Create README Documentation

**Files:**
- Create: `README.md`

**Step 1: Create README**

Create `README.md`:
```markdown
# aiTerm

An AI-First local terminal with Ollama and OpenAI-compatible cloud API support.

## Features

- 🖥️ **Native Terminal** - Full-featured terminal emulator powered by xterm.js
- 🤖 **AI Assistant** - Context-aware AI help for DevOps tasks
- 🔒 **Privacy First** - Local LLM support via Ollama
- ☁️ **Cloud Option** - Switch to OpenAI-compatible APIs when needed
- ✅ **Safe Execution** - Command confirmation before running AI suggestions

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://rustlang.org/) >= 1.70
- [Ollama](https://ollama.ai/) (optional, for local LLM)

### Development

\`\`\`bash
# Clone the repository
git clone https://github.com/yourname/aiTerm.git
cd aiTerm

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
\`\`\`

### Build

\`\`\`bash
# Build for production
pnpm tauri build
\`\`\`

## Configuration

Configuration is stored in `~/.aiterm/config.json`:

\`\`\`json
{
  "llm": {
    "provider": "ollama",
    "ollama": {
      "host": "http://localhost:11434",
      "model": "llama3.2"
    },
    "openai": {
      "api_base": "https://api.openai.com/v1",
      "model": "gpt-4o-mini"
    }
  },
  "terminal": {
    "shell": "auto",
    "font_size": 14,
    "theme": "dark"
  }
}
\`\`\`

API keys are stored securely in your OS keychain.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Space` | Focus AI panel |
| `Ctrl+,` | Open settings |
| `y` | Execute suggested command |
| `n` | Cancel command |
| `e` | Edit command |

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Pinia
- **Terminal**: xterm.js
- **Desktop**: Tauri 2.0
- **Backend**: Rust
- **PTY**: portable-pty
- **LLM**: Ollama API + OpenAI-compatible APIs

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README documentation"
```

---

## Phase 6: Testing & Polish (M6)

### Task 6.1: Add Unit Tests for Rust Backend

**Files:**
- Create: `src-tauri/src/pty/mod.rs` tests
- Create: `src-tauri/src/context/mod.rs` tests
- Create: `src-tauri/src/config/mod.rs` tests

**Step 1: Add tests to context module**

Add to `src-tauri/src/context/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_push() {
        let mut manager = ContextManager::new(10);
        manager.push_input("ls -la".to_string());
        manager.push_output("total 0".to_string());

        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_context_manager_max_lines() {
        let mut manager = ContextManager::new(3);
        manager.push_input("cmd1".to_string());
        manager.push_input("cmd2".to_string());
        manager.push_input("cmd3".to_string());
        manager.push_input("cmd4".to_string());

        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_context_manager_build_context() {
        let mut manager = ContextManager::new(10);
        manager.push_input("ls".to_string());
        manager.push_output("file1.txt".to_string());

        let context = manager.build_context(1000);
        assert!(context.contains("ls"));
        assert!(context.contains("file1.txt"));
    }
}
```

**Step 2: Run tests**

```bash
cd src-tauri && cargo test
```

**Step 3: Commit**

```bash
git add src-tauri/src/context/mod.rs
git commit -m "test: add unit tests for context manager"
```

---

### Task 6.2: Add Frontend Component Tests

**Files:**
- Create: `vitest.config.ts`
- Create: `src/components/__tests__/Terminal.spec.ts`

**Step 1: Install test dependencies**

```bash
pnpm add -D vitest @vue/test-utils happy-dom
```

**Step 2: Create vitest config**

Create `vitest.config.ts`:
```typescript
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'happy-dom',
    globals: true,
  },
});
```

**Step 3: Add test script to package.json**

```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest run --coverage"
  }
}
```

**Step 4: Commit**

```bash
git add vitest.config.ts package.json
git commit -m "test: setup vitest for frontend testing"
```

---

### Task 6.3: Final Integration Test

**Step 1: Run full build**

```bash
pnpm tauri build
```

**Step 2: Test the built application**

- Launch the app
- Verify terminal works
- Verify AI chat works with Ollama
- Verify settings persistence
- Verify command confirmation flow

**Step 3: Final commit**

```bash
git add .
git commit -m "chore: final integration test and polish"
```

---

## Summary

This plan covers the complete implementation of aiTerm across 6 phases:

| Phase | Tasks | Key Deliverables |
|-------|-------|------------------|
| M1 | 6 | Project skeleton, dependencies, stores |
| M2 | 3 | PTY manager, terminal component |
| M3 | 4 | Config, context, LLM client |
| M4 | 4 | AI chat, confirmation, settings UI |
| M5 | 3 | Context capture, error handling, docs |
| M6 | 3 | Tests, integration, polish |

**Total: 23 tasks**
