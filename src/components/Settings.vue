<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore, type AppSettings } from '../stores/settings'

// Props
defineProps<{
  visible: boolean
}>()

// Emits
const emit = defineEmits<{
  close: []
  saved: []
}>()

// Store
const settingsStore = useSettingsStore()

// Local state for editing
const localSettings = ref<AppSettings>(JSON.parse(JSON.stringify(settingsStore.settings)))
const activeTab = ref<'llm' | 'terminal' | 'about'>('llm')

// Ollama state
const ollamaModels = ref<string[]>([])
const ollamaLoading = ref(false)
const ollamaConnected = ref(false)
const ollamaTesting = ref(false)

// OpenAI state
const openaiModels = ref<string[]>([])
const openaiLoading = ref(false)
const openaiConnected = ref(false)
const openaiTesting = ref(false)
const apiKeyInput = ref('')
const showApiKey = ref(false)
const savingApiKey = ref(false)

// Status messages
const statusMessage = ref('')
const statusType = ref<'success' | 'error' | 'info'>('info')

// Available shells
const availableShells = [
  { value: 'auto', label: 'Auto-detect' },
  { value: '/bin/zsh', label: 'Zsh' },
  { value: '/bin/bash', label: 'Bash' },
  { value: '/bin/sh', label: 'Sh' },
]

// App info
const appInfo = {
  name: 'aiTerm',
  version: '0.1.0',
  description: 'AI-First Local Terminal - An intelligent terminal with AI-powered assistance',
  github: 'https://github.com/luopeng/aiTerm'
}

// Watch for visibility changes to reload settings
watch(() => settingsStore.settings, (newSettings) => {
  localSettings.value = JSON.parse(JSON.stringify(newSettings))
}, { deep: true })

// Reset local settings when modal opens (can be called from parent)
function _resetLocalSettings(): void {
  localSettings.value = JSON.parse(JSON.stringify(settingsStore.settings))
  statusMessage.value = ''
}
void _resetLocalSettings // Suppress unused warning

// URL validation for SSRF protection
function validateUrl(url: string, type: 'ollama' | 'openai'): { valid: boolean; error?: string } {
  if (!url || url.trim() === '') {
    return { valid: false, error: 'URL is required' }
  }

  try {
    const parsed = new URL(url)

    // Only allow http and https protocols
    if (!['http:', 'https:'].includes(parsed.protocol)) {
      return { valid: false, error: 'Only http:// and https:// protocols are allowed' }
    }

    // Block private IP ranges for external APIs (OpenAI)
    if (type === 'openai') {
      const hostname = parsed.hostname

      // Block localhost variations
      if (['localhost', '127.0.0.1', '::1'].includes(hostname)) {
        return { valid: false, error: 'External API should not point to localhost' }
      }

      // Block private IP ranges (basic check)
      const privateIpPatterns = [
        /^10\./,                    // 10.0.0.0/8
        /^172\.(1[6-9]|2[0-9]|3[0-1])\./, // 172.16.0.0/12
        /^192\.168\./,              // 192.168.0.0/16
        /^169\.254\./,              // Link-local
      ]

      for (const pattern of privateIpPatterns) {
        if (pattern.test(hostname)) {
          return { valid: false, error: 'Private IP addresses are not allowed for external APIs' }
        }
      }
    }

    return { valid: true }
  } catch {
    return { valid: false, error: 'Invalid URL format' }
  }
}

// Show status message
function showStatus(message: string, type: 'success' | 'error' | 'info'): void {
  statusMessage.value = message
  statusType.value = type
  setTimeout(() => {
    statusMessage.value = ''
  }, 3000)
}

// Fetch Ollama models
async function fetchOllamaModels(): Promise<void> {
  ollamaLoading.value = true
  try {
    const models = await invoke<string[]>('ai_list_models', {
      provider: 'ollama',
      host: localSettings.value.llm.ollama.host
    })
    ollamaModels.value = models
  } catch (error) {
    console.error('Failed to fetch Ollama models:', error)
    ollamaModels.value = []
  } finally {
    ollamaLoading.value = false
  }
}

// Test Ollama connection
async function testOllamaConnection(): Promise<void> {
  // Validate URL for SSRF protection
  const validation = validateUrl(localSettings.value.llm.ollama.host, 'ollama')
  if (!validation.valid) {
    showStatus('Invalid host: ' + (validation.error || 'Unknown error'), 'error')
    return
  }

  ollamaTesting.value = true
  try {

    const result = await invoke<boolean>('ai_test_connection', {
      provider: 'ollama',
      host: localSettings.value.llm.ollama.host
    })
    ollamaConnected.value = result
    if (result) {
      showStatus('Ollama connection successful!', 'success')
      await fetchOllamaModels()
    } else {
      showStatus('Failed to connect to Ollama', 'error')
    }
  } catch (error) {
    console.error('Ollama connection test failed:', error)
    ollamaConnected.value = false
    showStatus('Connection test failed: ' + (error as Error).message, 'error')
  } finally {
    ollamaTesting.value = false
  }
}

// Fetch OpenAI models
async function fetchOpenaiModels(): Promise<void> {
  openaiLoading.value = true
  try {
    const models = await invoke<string[]>('ai_list_models', {
      provider: 'openai',
      apiBase: localSettings.value.llm.openai.apiBase
    })
    openaiModels.value = models
  } catch (error) {
    console.error('Failed to fetch OpenAI models:', error)
    openaiModels.value = []
  } finally {
    openaiLoading.value = false
  }
}

// Test OpenAI connection
async function testOpenaiConnection(): Promise<void> {
  // Validate API base URL for SSRF protection
  const validation = validateUrl(localSettings.value.llm.openai.apiBase, 'openai')
  if (!validation.valid) {
    showStatus('Invalid API URL: ' + validation.error, 'error')
    return
  }

  openaiTesting.value = true
  try {
    const result = await invoke<boolean>('ai_test_connection', {
      provider: 'openai',
      apiBase: localSettings.value.llm.openai.apiBase
    })
    openaiConnected.value = result
    if (result) {
      showStatus('OpenAI connection successful!', 'success')
      await fetchOpenaiModels()
    } else {
      showStatus('Failed to connect to OpenAI', 'error')
    }
  } catch (error) {
    console.error('OpenAI connection test failed:', error)
    openaiConnected.value = false
    showStatus('Connection test failed: ' + (error as Error).message, 'error')
  } finally {
    openaiTesting.value = false
  }
}

// Save API key to keyring
async function saveApiKey(): Promise<void> {
  if (!apiKeyInput.value.trim()) {
    showStatus('Please enter an API key', 'error')
    return
  }

  savingApiKey.value = true
  try {
    await invoke('save_api_key', {
      provider: 'openai',
      apiKey: apiKeyInput.value
    })
    localSettings.value.llm.openai.hasApiKey = true
    showStatus('API key saved successfully!', 'success')
    apiKeyInput.value = ''
  } catch (error) {
    console.error('Failed to save API key:', error)
    showStatus('Failed to save API key: ' + (error as Error).message, 'error')
  } finally {
    savingApiKey.value = false
  }
}

// Save settings
async function saveSettings(): Promise<void> {
  try {
    // Update store
    settingsStore.settings = JSON.parse(JSON.stringify(localSettings.value))

    // Save to backend
    await settingsStore.saveSettings()

    showStatus('Settings saved successfully!', 'success')
    emit('saved')
  } catch (error) {
    console.error('Failed to save settings:', error)
    showStatus('Failed to save settings: ' + (error as Error).message, 'error')
  }
}

// Reset to defaults
function resetToDefaults(): void {
  localSettings.value = {
    llm: {
      provider: 'ollama',
      ollama: {
        host: 'http://localhost:11434',
        model: 'llama3.2'
      },
      openai: {
        apiBase: 'https://api.openai.com/v1',
        model: 'gpt-4o-mini',
        hasApiKey: false
      }
    },
    terminal: {
      shell: 'auto',
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, monospace',
      theme: 'dark'
    },
    context: {
      maxLines: 500,
      maxTokens: 4096
    }
  }
  showStatus('Settings reset to defaults', 'info')
}

// Close modal
function closeModal(): void {
  emit('close')
}

// Handle overlay click
function handleOverlayClick(event: MouseEvent): void {
  if (event.target === event.currentTarget) {
    closeModal()
  }
}

// Load models on mount and when provider changes
onMounted(() => {
  if (localSettings.value.llm.provider === 'ollama') {
    fetchOllamaModels()
  } else {
    fetchOpenaiModels()
  }
})

// Watch provider changes
watch(() => localSettings.value.llm.provider, (newProvider) => {
  if (newProvider === 'ollama') {
    fetchOllamaModels()
  } else {
    fetchOpenaiModels()
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="visible"
        class="settings-overlay"
        @click="handleOverlayClick"
      >
        <div class="settings-modal">
          <!-- Header -->
          <div class="settings-header">
            <h2>Settings</h2>
            <button class="close-btn" @click="closeModal" aria-label="Close">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>

          <!-- Tab Bar -->
          <div class="tab-bar">
            <button
              :class="['tab', { active: activeTab === 'llm' }]"
              @click="activeTab = 'llm'"
            >
              LLM
            </button>
            <button
              :class="['tab', { active: activeTab === 'terminal' }]"
              @click="activeTab = 'terminal'"
            >
              Terminal
            </button>
            <button
              :class="['tab', { active: activeTab === 'about' }]"
              @click="activeTab = 'about'"
            >
              About
            </button>
          </div>

          <!-- Status Message -->
          <Transition name="fade">
            <div v-if="statusMessage" :class="['status-message', statusType]">
              {{ statusMessage }}
            </div>
          </Transition>

          <!-- Content -->
          <div class="settings-content">
            <!-- LLM Tab -->
            <div v-show="activeTab === 'llm'" class="tab-content">
              <!-- Provider Selection -->
              <div class="form-group">
                <label>Provider</label>
                <div class="provider-buttons">
                  <button
                    :class="['provider-btn', { active: localSettings.llm.provider === 'ollama' }]"
                    @click="localSettings.llm.provider = 'ollama'"
                  >
                    Ollama
                  </button>
                  <button
                    :class="['provider-btn', { active: localSettings.llm.provider === 'openai' }]"
                    @click="localSettings.llm.provider = 'openai'"
                  >
                    OpenAI
                  </button>
                </div>
              </div>

              <!-- Ollama Settings -->
              <div v-if="localSettings.llm.provider === 'ollama'" class="provider-settings">
                <div class="form-group">
                  <label>Host</label>
                  <input
                    v-model="localSettings.llm.ollama.host"
                    type="text"
                    placeholder="http://localhost:11434"
                  />
                </div>

                <div class="form-group">
                  <label>Model</label>
                  <input
                    v-model="localSettings.llm.ollama.model"
                    type="text"
                    placeholder="e.g., llama3.2, mistral, codellama"
                    list="ollama-models"
                  />
                  <datalist id="ollama-models">
                    <option v-for="model in ollamaModels" :key="model" :value="model" />
                  </datalist>
                  <small class="hint" v-if="ollamaModels.length > 0">
                    Available: {{ ollamaModels.slice(0, 5).join(', ') }}{{ ollamaModels.length > 5 ? '...' : '' }}
                  </small>
                </div>

                <div class="form-group">
                  <label>Connection Status</label>
                  <div class="connection-status">
                    <span :class="['status-indicator', ollamaConnected ? 'connected' : 'disconnected']"></span>
                    <span>{{ ollamaConnected ? 'Connected' : 'Disconnected' }}</span>
                    <button
                      class="test-btn"
                      :disabled="ollamaTesting"
                      @click="testOllamaConnection"
                    >
                      {{ ollamaTesting ? 'Testing...' : 'Test Connection' }}
                    </button>
                  </div>
                </div>
              </div>

              <!-- OpenAI Settings -->
              <div v-if="localSettings.llm.provider === 'openai'" class="provider-settings">
                <div class="form-group">
                  <label>API Base</label>
                  <input
                    v-model="localSettings.llm.openai.apiBase"
                    type="text"
                    placeholder="https://api.openai.com/v1"
                  />
                </div>

                <div class="form-group">
                  <label>Model</label>
                  <input
                    v-model="localSettings.llm.openai.model"
                    type="text"
                    placeholder="e.g., gpt-4o-mini, gpt-4o, gpt-4-turbo"
                    list="openai-models"
                  />
                  <datalist id="openai-models">
                    <option v-for="model in openaiModels" :key="model" :value="model" />
                  </datalist>
                  <small class="hint" v-if="openaiModels.length > 0">
                    Available: {{ openaiModels.slice(0, 5).join(', ') }}{{ openaiModels.length > 5 ? '...' : '' }}
                  </small>
                </div>

                <div class="form-group">
                  <label>API Key</label>
                  <div class="api-key-input">
                    <input
                      v-model="apiKeyInput"
                      :type="showApiKey ? 'text' : 'password'"
                      placeholder="Enter your API key"
                    />
                    <button
                      class="toggle-visibility"
                      @click="showApiKey = !showApiKey"
                      :aria-label="showApiKey ? 'Hide API key' : 'Show API key'"
                    >
                      <svg v-if="showApiKey" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                        <line x1="1" y1="1" x2="23" y2="23"></line>
                      </svg>
                      <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                        <circle cx="12" cy="12" r="3"></circle>
                      </svg>
                    </button>
                    <button
                      class="save-key-btn"
                      :disabled="savingApiKey || !apiKeyInput.trim()"
                      @click="saveApiKey"
                    >
                      {{ savingApiKey ? 'Saving...' : 'Save' }}
                    </button>
                  </div>
                  <div class="api-key-status">
                    <span :class="['status-indicator', localSettings.llm.openai.hasApiKey ? 'connected' : 'disconnected']"></span>
                    <span>{{ localSettings.llm.openai.hasApiKey ? 'API key saved' : 'No API key saved' }}</span>
                  </div>
                </div>

                <div class="form-group">
                  <label>Connection Status</label>
                  <div class="connection-status">
                    <span :class="['status-indicator', openaiConnected ? 'connected' : 'disconnected']"></span>
                    <span>{{ openaiConnected ? 'Connected' : 'Disconnected' }}</span>
                    <button
                      class="test-btn"
                      :disabled="openaiTesting || !localSettings.llm.openai.hasApiKey"
                      @click="testOpenaiConnection"
                    >
                      {{ openaiTesting ? 'Testing...' : 'Test Connection' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Terminal Tab -->
            <div v-show="activeTab === 'terminal'" class="tab-content">
              <div class="form-group">
                <label>Shell</label>
                <select v-model="localSettings.terminal.shell">
                  <option v-for="shell in availableShells" :key="shell.value" :value="shell.value">
                    {{ shell.label }}
                  </option>
                </select>
              </div>

              <div class="form-group">
                <label>Font Size: {{ localSettings.terminal.fontSize }}px</label>
                <input
                  v-model.number="localSettings.terminal.fontSize"
                  type="range"
                  min="10"
                  max="24"
                  step="1"
                />
                <div class="range-labels">
                  <span>10px</span>
                  <span>24px</span>
                </div>
              </div>

              <div class="form-group">
                <label>Font Family</label>
                <input
                  v-model="localSettings.terminal.fontFamily"
                  type="text"
                  placeholder="Menlo, Monaco, monospace"
                />
              </div>

              <div class="form-group">
                <label>Theme</label>
                <div class="theme-toggle">
                  <button
                    :class="['theme-btn', { active: localSettings.terminal.theme === 'dark' }]"
                    @click="localSettings.terminal.theme = 'dark'"
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                    </svg>
                    Dark
                  </button>
                  <button
                    :class="['theme-btn', { active: localSettings.terminal.theme === 'light' }]"
                    @click="localSettings.terminal.theme = 'light'"
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="5"></circle>
                      <line x1="12" y1="1" x2="12" y2="3"></line>
                      <line x1="12" y1="21" x2="12" y2="23"></line>
                      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                      <line x1="1" y1="12" x2="3" y2="12"></line>
                      <line x1="21" y1="12" x2="23" y2="12"></line>
                      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                    </svg>
                    Light
                  </button>
                </div>
              </div>
            </div>

            <!-- About Tab -->
            <div v-show="activeTab === 'about'" class="tab-content about-tab">
              <div class="app-icon">
                <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                  <line x1="8" y1="21" x2="16" y2="21"></line>
                  <line x1="12" y1="17" x2="12" y2="21"></line>
                  <path d="M6 8h.01M6 12h.01M6 16h.01"></path>
                  <path d="M10 8h8M10 12h8M10 16h4"></path>
                </svg>
              </div>
              <h3>{{ appInfo.name }}</h3>
              <p class="version">Version {{ appInfo.version }}</p>
              <p class="description">{{ appInfo.description }}</p>
              <div class="links">
                <a :href="appInfo.github" target="_blank" rel="noopener noreferrer">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                  </svg>
                  GitHub
                </a>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="settings-footer">
            <button class="btn btn-secondary" @click="resetToDefaults">
              Reset to Defaults
            </button>
            <div class="footer-actions">
              <button class="btn btn-ghost" @click="closeModal">
                Cancel
              </button>
              <button class="btn btn-primary" @click="saveSettings">
                Save
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.settings-modal {
  background-color: #1e1e2e;
  border-radius: 12px;
  width: 90%;
  max-width: 560px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
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
  font-weight: 600;
  color: #cdd6f4;
}

.close-btn {
  background: none;
  border: none;
  color: #6c7086;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.close-btn:hover {
  color: #cdd6f4;
  background-color: #313244;
}

.tab-bar {
  display: flex;
  border-bottom: 1px solid #313244;
  padding: 0 20px;
}

.tab {
  background: none;
  border: none;
  padding: 12px 16px;
  font-size: 14px;
  color: #6c7086;
  cursor: pointer;
  position: relative;
  transition: color 0.15s ease;
}

.tab:hover {
  color: #a6adc8;
}

.tab.active {
  color: #89b4fa;
}

.tab.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 16px;
  right: 16px;
  height: 2px;
  background-color: #89b4fa;
  border-radius: 1px;
}

.status-message {
  margin: 12px 20px 0;
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
}

.status-message.success {
  background-color: rgba(166, 227, 161, 0.15);
  color: #a6e3a1;
  border: 1px solid rgba(166, 227, 161, 0.3);
}

.status-message.error {
  background-color: rgba(243, 139, 168, 0.15);
  color: #f38ba8;
  border: 1px solid rgba(243, 139, 168, 0.3);
}

.status-message.info {
  background-color: rgba(137, 180, 250, 0.15);
  color: #89b4fa;
  border: 1px solid rgba(137, 180, 250, 0.3);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: #a6adc8;
}

.form-group input[type="text"],
.form-group input[type="password"],
.form-group select {
  background-color: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  padding: 10px 12px;
  font-size: 14px;
  color: #cdd6f4;
  outline: none;
  transition: border-color 0.15s ease;
}

.form-group input:focus,
.form-group select:focus {
  border-color: #89b4fa;
}

.form-group input::placeholder {
  color: #6c7086;
}

.form-group select {
  cursor: pointer;
}

.form-group select option {
  background-color: #313244;
  color: #cdd6f4;
}

.form-group .hint {
  font-size: 11px;
  color: #6c7086;
  margin-top: 2px;
}

.provider-buttons {
  display: flex;
  gap: 8px;
}

.provider-btn {
  flex: 1;
  padding: 10px 16px;
  background-color: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  color: #a6adc8;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.provider-btn:hover {
  background-color: #45475a;
  color: #cdd6f4;
}

.provider-btn.active {
  background-color: #89b4fa;
  border-color: #89b4fa;
  color: #1e1e2e;
}

.select-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.select-wrapper select {
  width: 100%;
  padding-right: 36px;
}

.loading-spinner {
  position: absolute;
  right: 12px;
  width: 16px;
  height: 16px;
  border: 2px solid #45475a;
  border-top-color: #89b4fa;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.status-indicator.connected {
  background-color: #a6e3a1;
  box-shadow: 0 0 8px rgba(166, 227, 161, 0.5);
}

.status-indicator.disconnected {
  background-color: #f38ba8;
}

.test-btn {
  margin-left: auto;
  padding: 6px 12px;
  background-color: #313244;
  border: 1px solid #45475a;
  border-radius: 4px;
  color: #a6adc8;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.test-btn:hover:not(:disabled) {
  background-color: #45475a;
  color: #cdd6f4;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.api-key-input {
  display: flex;
  gap: 8px;
}

.api-key-input input {
  flex: 1;
}

.toggle-visibility {
  background: none;
  border: 1px solid #45475a;
  border-radius: 6px;
  padding: 8px;
  color: #6c7086;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.toggle-visibility:hover {
  color: #cdd6f4;
  background-color: #45475a;
}

.save-key-btn {
  padding: 8px 14px;
  background-color: #89b4fa;
  border: none;
  border-radius: 6px;
  color: #1e1e2e;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.save-key-btn:hover:not(:disabled) {
  background-color: #b4befe;
}

.save-key-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.api-key-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
  font-size: 12px;
  color: #6c7086;
}

input[type="range"] {
  -webkit-appearance: none;
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: #313244;
  outline: none;
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #89b4fa;
  cursor: pointer;
  transition: transform 0.15s ease;
}

input[type="range"]::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.range-labels {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: #6c7086;
  margin-top: 4px;
}

.theme-toggle {
  display: flex;
  gap: 8px;
}

.theme-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  background-color: #313244;
  border: 1px solid #45475a;
  border-radius: 6px;
  color: #a6adc8;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-btn:hover {
  background-color: #45475a;
  color: #cdd6f4;
}

.theme-btn.active {
  background-color: #89b4fa;
  border-color: #89b4fa;
  color: #1e1e2e;
}

.about-tab {
  text-align: center;
  padding: 20px 0;
}

.app-icon {
  margin-bottom: 16px;
  color: #89b4fa;
}

.about-tab h3 {
  margin: 0 0 8px;
  font-size: 24px;
  font-weight: 600;
  color: #cdd6f4;
}

.about-tab .version {
  margin: 0 0 16px;
  font-size: 14px;
  color: #6c7086;
}

.about-tab .description {
  margin: 0 0 24px;
  font-size: 14px;
  color: #a6adc8;
  line-height: 1.5;
  max-width: 360px;
  margin-left: auto;
  margin-right: auto;
}

.links {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.links a {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: #89b4fa;
  text-decoration: none;
  font-size: 14px;
  padding: 8px 16px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.links a:hover {
  background-color: rgba(137, 180, 250, 0.15);
}

.settings-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-top: 1px solid #313244;
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 10px 18px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary {
  background-color: #89b4fa;
  border: none;
  color: #1e1e2e;
}

.btn-primary:hover {
  background-color: #b4befe;
}

.btn-secondary {
  background-color: transparent;
  border: 1px solid #45475a;
  color: #a6adc8;
}

.btn-secondary:hover {
  background-color: #313244;
  color: #cdd6f4;
}

.btn-ghost {
  background-color: transparent;
  border: none;
  color: #6c7086;
}

.btn-ghost:hover {
  color: #cdd6f4;
}

/* Modal transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .settings-modal,
.modal-leave-active .settings-modal {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .settings-modal,
.modal-leave-to .settings-modal {
  transform: scale(0.95) translateY(-20px);
  opacity: 0;
}

/* Fade transition for status message */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
