<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import Terminal from './components/Terminal.vue'
import AIChat from './components/AIChat.vue'
import CmdConfirm from './components/CmdConfirm.vue'
import Settings from './components/Settings.vue'
import { useSettingsStore } from './stores/settings'
import { useAIStore } from './stores/ai'
import { useTerminalStore } from './stores/terminal'

// Stores
const settings = useSettingsStore()
const aiStore = useAIStore()
const terminalStore = useTerminalStore()

// Component refs
const terminalRef = ref<InstanceType<typeof Terminal> | null>(null)

// Modal state
const showSettings = ref(false)
const showCmdConfirm = ref(false)
const pendingCommand = ref('')

// Context stats
const contextStats = ref<{
  entries_in_buffer: number
  pending_summary_count: number
  total_entries: number
  has_summary: boolean
  summary_entry_count: number
} | null>(null)

// PTY state
const ptyId = ref<string | null>(null)
const ptyStatus = ref<'disconnected' | 'connecting' | 'connected' | 'exited'>('disconnected')

// Connection status
const ollamaConnected = ref(false)
const openaiConnected = ref(false)

// Computed
const aiConnected = computed(() => {
  return settings.settings.llm.provider === 'ollama' ? ollamaConnected.value : openaiConnected.value
})

// Status text for debugging (can be used in future status displays)
const _statusText = computed(() => {
  const parts: string[] = []

  // PTY status
  if (ptyStatus.value === 'connected') {
    parts.push('PTY: Connected')
  } else if (ptyStatus.value === 'connecting') {
    parts.push('PTY: Connecting...')
  } else if (ptyStatus.value === 'exited') {
    parts.push('PTY: Exited')
  } else {
    parts.push('PTY: Disconnected')
  }

  // AI status
  const provider = settings.settings.llm.provider
  if (aiConnected.value) {
    parts.push(`${provider === 'ollama' ? 'Ollama' : 'OpenAI'}: Connected`)
  } else {
    parts.push(`${provider === 'ollama' ? 'Ollama' : 'OpenAI'}: Disconnected`)
  }

  return parts.join(' | ')
})
void _statusText // Suppress unused warning

// Tauri event unlisteners
let unlisteners: UnlistenFn[] = []

// Type declaration for Tauri window object
declare global {
  interface Window {
    __TAURI__?: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>
    }
  }
}

// ============ PTY Management ============

async function createPTY(): Promise<void> {
  ptyStatus.value = 'connecting'

  try {
    // Get terminal size
    const size = terminalRef.value?.getSize()
    const cols = size?.cols ?? 80
    const rows = size?.rows ?? 24

    // Generate a unique ID for this PTY session
    const id = `pty_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

    // Create PTY session (backend expects id parameter)
    await invoke('pty_create', {
      id,
      cols,
      rows
    })

    ptyId.value = id
    ptyStatus.value = 'connected'

    console.log('PTY created with ID:', id)
  } catch (error) {
    console.error('Failed to create PTY:', error)
    ptyStatus.value = 'disconnected'
  }
}

async function writeToPTY(data: string): Promise<void> {
  if (!ptyId.value) return

  try {
    await invoke('pty_write', {
      id: ptyId.value,
      data
    })
  } catch (error) {
    console.error('Failed to write to PTY:', error)
  }
}

async function resizePTY(cols: number, rows: number): Promise<void> {
  if (!ptyId.value) return

  try {
    await invoke('pty_resize', {
      id: ptyId.value,
      cols,
      rows
    })
  } catch (error) {
    console.error('Failed to resize PTY:', error)
  }
}

// ============ Context Management ============

async function pushToContext(type: 'input' | 'output', content: string): Promise<void> {
  try {
    if (type === 'input') {
      await invoke('context_push_input', { content })
    } else {
      await invoke('context_push_output', { content })
    }

    // Check if summarization is needed after each push
    await checkAndSummarize()
  } catch (error) {
    console.error('Failed to push to context:', error)
  }
}

// Check if context needs summarization and trigger if needed
async function checkAndSummarize(): Promise<void> {
  try {
    const hasPending = await invoke<boolean>('context_has_pending_summary')
    if (hasPending) {
      console.log('Context pending summarization, triggering...')
      await invoke('context_summarize')
      console.log('Context summarization completed')
    }

    // Update stats
    await updateContextStats()
  } catch (error) {
    console.error('Failed to summarize context:', error)
  }
}

// Update context stats for display
async function updateContextStats(): Promise<void> {
  try {
    contextStats.value = await invoke<{
      entries_in_buffer: number
      pending_summary_count: number
      total_entries: number
      has_summary: boolean
      summary_entry_count: number
    }>('context_stats')
  } catch (error) {
    console.error('Failed to get context stats:', error)
  }
}

// Get recent errors from context (for future error highlighting feature)
async function _getContextErrors(): Promise<string[]> {
  try {
    return await invoke<string[]>('context_get_errors')
  } catch (error) {
    console.error('Failed to get context errors:', error)
    return []
  }
}
void _getContextErrors // Suppress unused warning

// ============ Event Handlers ============

// Handle terminal ready
async function onTerminalReady(): Promise<void> {
  console.log('Terminal is ready')
  terminalStore.isReady = true

  // Create PTY session
  await createPTY()
}

// Handle terminal data (user input)
async function onTerminalData(data: string): Promise<void> {
  // Send to PTY
  await writeToPTY(data)

  // Push to context for AI
  await pushToContext('input', data)
  terminalStore.addOutput(data, 'input')
}

// Handle terminal resize
async function onTerminalResize(cols: number, rows: number): Promise<void> {
  console.log('Terminal resized:', cols, rows)
  await resizePTY(cols, rows)
}

// Handle command execution request from AI
function onExecuteCommand(command: string): void {
  pendingCommand.value = command
  showCmdConfirm.value = true
}

// Handle command confirmation
async function onCommandConfirm(): Promise<void> {
  const command = pendingCommand.value
  if (!command) return

  // Add newline if not present
  const data = command.endsWith('\n') ? command : command + '\n'
  await writeToPTY(data)
  await pushToContext('input', data)
  terminalStore.addOutput(data, 'input')

  // Focus terminal
  terminalRef.value?.focus()
}

// Handle command edit
function onCommandEdit(command: string): void {
  // Close confirm modal
  showCmdConfirm.value = false

  // Add message to AI chat about edited command
  aiStore.addMessage('user', `Edited command: ${command}`)

  // Re-execute the edited command
  pendingCommand.value = command
  showCmdConfirm.value = true
}

// Handle command cancel
function onCommandCancel(): void {
  showCmdConfirm.value = false
  pendingCommand.value = ''
}

// Handle settings saved
async function onSettingsSaved(): Promise<void> {
  // Reload settings
  await settings.loadSettings()

  // Update AI provider in frontend
  aiStore.setProvider(settings.settings.llm.provider)

  // Re-initialize AI provider in backend with new settings
  await initAIProvider()

  // Test connections
  await testAIConnection()
}

// Handle keyboard shortcuts
function handleKeydown(event: KeyboardEvent): void {
  // Cmd+, or Ctrl+, for settings
  if ((event.metaKey || event.ctrlKey) && event.key === ',') {
    event.preventDefault()
    showSettings.value = true
  }

  // Escape to close modals
  if (event.key === 'Escape') {
    if (showCmdConfirm.value) {
      showCmdConfirm.value = false
    }
  }

  // ? to show help (optional)
  if (event.key === '?' && !event.metaKey && !event.ctrlKey) {
    // Could show a help modal here
  }
}

// Test AI connection
async function testAIConnection(): Promise<void> {
  try {
    const result = await invoke<boolean>('ai_test_connection', {
      provider: settings.settings.llm.provider,
      host: settings.settings.llm.provider === 'ollama'
        ? settings.settings.llm.ollama.host
        : undefined,
      apiBase: settings.settings.llm.provider === 'openai'
        ? settings.settings.llm.openai.apiBase
        : undefined
    })

    if (settings.settings.llm.provider === 'ollama') {
      ollamaConnected.value = result
    } else {
      openaiConnected.value = result
    }
  } catch (error) {
    console.error('AI connection test failed:', error)
  }
}

// Initialize AI provider in backend
async function initAIProvider(): Promise<void> {
  try {
    await invoke('ai_set_provider', {
      provider: settings.settings.llm.provider,
      apiKey: undefined // For OpenAI, API key is stored in keychain
    })
    console.log('AI provider initialized:', settings.settings.llm.provider)
  } catch (error) {
    console.error('Failed to initialize AI provider:', error)
  }
}

// Set up Tauri event listeners
async function setupEventListeners(): Promise<void> {
  // Listen for PTY output
  const unlistenOutput = await listen<{ id: string; data: string }>('pty_output', (event) => {
    if (event.payload.id === ptyId.value) {
      terminalRef.value?.write(event.payload.data)
      pushToContext('output', event.payload.data)
      terminalStore.addOutput(event.payload.data, 'output')
    }
  })
  unlisteners.push(unlistenOutput)

  // Listen for PTY exit
  const unlistenExit = await listen<{ id: string; exit_code: number }>('pty_exit', (event) => {
    if (event.payload.id === ptyId.value) {
      ptyStatus.value = 'exited'
      terminalRef.value?.writeln(`\r\n\x1b[33m[Process exited with code ${event.payload.exit_code}]\x1b[0m`)
    }
  })
  unlisteners.push(unlistenExit)

  // AI stream events are handled in AIChat component
}

// Clean up event listeners
function cleanupEventListeners(): void {
  unlisteners.forEach((unlisten) => unlisten())
  unlisteners = []
}

// Lifecycle hooks
onMounted(async () => {
  // Load settings
  await settings.loadSettings()

  // Set AI provider in frontend store
  aiStore.setProvider(settings.settings.llm.provider)

  // Initialize AI provider in backend
  await initAIProvider()

  // Set up event listeners
  await setupEventListeners()

  // Test AI connection
  await testAIConnection()

  // Add keyboard listener
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  // Clean up
  cleanupEventListeners()
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="app" :class="settings.theme">
    <div class="main-content">
      <!-- Terminal Panel -->
      <div class="terminal-panel">
        <Terminal
          ref="terminalRef"
          :fontSize="settings.settings.terminal.fontSize"
          :fontFamily="settings.settings.terminal.fontFamily"
          :theme="settings.settings.terminal.theme"
          @ready="onTerminalReady"
          @data="onTerminalData"
          @resize="onTerminalResize"
        />
      </div>

      <!-- AI Chat Panel -->
      <AIChat @executeCommand="onExecuteCommand" />
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-left">
        <span class="status-item" :class="{ connected: ptyStatus === 'connected' }">
          <span class="status-dot"></span>
          {{ ptyStatus === 'connected' ? 'PTY' : ptyStatus }}
        </span>
        <span class="status-item" :class="{ connected: aiConnected }">
          <span class="status-dot"></span>
          {{ settings.settings.llm.provider === 'ollama' ? 'Ollama' : 'OpenAI' }}
        </span>
      </div>
      <div class="status-center">
        <span class="model-name">
          {{ settings.settings.llm.provider === 'ollama'
            ? settings.settings.llm.ollama.model
            : settings.settings.llm.openai.model }}
        </span>
        <span v-if="contextStats" class="context-stats" :title="`Buffer: ${contextStats.entries_in_buffer}, Summary: ${contextStats.summary_entry_count} entries`">
          <span class="context-icon">📝</span>
          {{ contextStats.entries_in_buffer }}{{ contextStats.has_summary ? '+' + contextStats.summary_entry_count : '' }}
        </span>
      </div>
      <div class="status-right">
        <button class="settings-btn" @click="showSettings = true" title="Settings (Cmd+,)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
          </svg>
        </button>
        <span class="shortcut-hint">Cmd+, Settings</span>
      </div>
    </div>

    <!-- Command Confirmation Modal -->
    <CmdConfirm
      :visible="showCmdConfirm"
      :command="pendingCommand"
      @confirm="onCommandConfirm"
      @cancel="onCommandCancel"
      @edit="onCommandEdit"
      @close="showCmdConfirm = false"
    />

    <!-- Settings Modal -->
    <Settings
      :visible="showSettings"
      @close="showSettings = false"
      @saved="onSettingsSaved"
    />
  </div>
</template>

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

.terminal-panel {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* Status Bar */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
  padding: 0 12px;
  background-color: #11111b;
  border-top: 1px solid #313244;
  font-size: 11px;
  color: #6c7086;
}

.app.light .status-bar {
  background-color: #dce0e8;
  border-top-color: #ccd0da;
  color: #6c6f85;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-center {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
}

.model-name {
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 10px;
  color: #a6adc8;
}

.app.light .model-name {
  color: #7c7f93;
}

.context-stats {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 10px;
  color: #89b4fa;
  padding: 2px 6px;
  background-color: #313244;
  border-radius: 4px;
  margin-left: 8px;
}

.context-icon {
  font-size: 10px;
}

.app.light .context-stats {
  background-color: #ccd0da;
  color: #1e66f5;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #f38ba8;
}

.status-item.connected .status-dot {
  background-color: #a6e3a1;
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  background: transparent;
  border: none;
  color: #6c7086;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.settings-btn:hover {
  background-color: #313244;
  color: #cdd6f4;
}

.app.light .settings-btn:hover {
  background-color: #ccd0da;
  color: #4c4f69;
}

.shortcut-hint {
  font-size: 10px;
  color: #45475a;
}

.app.light .shortcut-hint {
  color: #9ca0b0;
}
</style>
