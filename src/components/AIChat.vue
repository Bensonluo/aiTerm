<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import DOMPurify from 'dompurify'
import { useAIStore } from '../stores/ai'

// Emits
const emit = defineEmits<{
  (e: 'executeCommand', command: string): void
}>()

// Store
const aiStore = useAIStore()

// Refs
const messagesContainer = ref<HTMLDivElement | null>(null)
const inputText = ref('')
const isCollapsed = ref(false)

// Computed
const panelWidth = computed(() => isCollapsed.value ? '40px' : '350px')
const providerBadge = computed(() => {
  return aiStore.provider === 'ollama' ? 'Ollama' : 'OpenAI'
})

// Tauri event unlisteners
let unlistenStream: (() => void) | null = null
let unlistenDone: (() => void) | null = null
let unlistenError: (() => void) | null = null

// Extract bash commands from content
function extractCommands(content: string): string[] {
  const commands: string[] = []
  const bashBlockRegex = /```bash\s*\n([\s\S]*?)```/g
  let match

  while ((match = bashBlockRegex.exec(content)) !== null) {
    const command = match[1].trim()
    // Only include single-line commands (no multiline scripts)
    const lines = command.split('\n').filter(line => line.trim() && !line.trim().startsWith('#'))
    if (lines.length === 1) {
      commands.push(lines[0].trim())
    }
  }

  return commands
}

// Simple markdown to HTML renderer with XSS protection
function renderMarkdown(content: string): string {
  let html = content

  // Escape HTML
  html = html.replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  // Code blocks
  html = html.replace(/```(\w*)\s*\n([\s\S]*?)```/g, (_, lang, code) => {
    const language = lang || 'plaintext'
    return `<div class="code-block"><div class="code-header"><span>${language}</span></div><pre><code class="language-${language}">${code.trim()}</code></pre></div>`
  })

  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>')

  // Bold
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')

  // Italic
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>')

  // Line breaks
  html = html.replace(/\n/g, '<br>')

  // Sanitize with DOMPurify for additional XSS protection
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ['div', 'span', 'pre', 'code', 'strong', 'em', 'br', 'p'],
    ALLOWED_ATTR: ['class']
  })
}

// Format timestamp for display
function formatTime(timestamp: Date): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })
}

// Send message
async function sendMessage(): Promise<void> {
  const content = inputText.value.trim()
  if (!content || aiStore.isStreaming) return

  inputText.value = ''
  await aiStore.sendMessage(content)
  scrollToBottom()
}

// Handle input keydown
function handleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    sendMessage()
  }
}

// Execute command
function executeCommand(command: string): void {
  emit('executeCommand', command)
}

// Toggle panel collapse
function toggleCollapse(): void {
  isCollapsed.value = !isCollapsed.value
}

// Scroll to bottom of messages
function scrollToBottom(): void {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

// Set up Tauri event listeners
async function setupEventListeners(): Promise<void> {
  try {
    // Listen for streaming content
    unlistenStream = await listen<{ content: string }>('ai_stream', (event) => {
      aiStore.appendStreamingContent(event.payload.content)
      scrollToBottom()
    })

    // Listen for streaming complete
    unlistenDone = await listen('ai_done', () => {
      aiStore.finishStreaming()
      scrollToBottom()
    })

    // Listen for errors
    unlistenError = await listen<{ error: string }>('ai_error', (event) => {
      aiStore.showError(event.payload.error)
    })
  } catch (error) {
    console.warn('Failed to set up Tauri event listeners:', error)
  }
}

// Clean up event listeners
function cleanupEventListeners(): void {
  if (unlistenStream) unlistenStream()
  if (unlistenDone) unlistenDone()
  if (unlistenError) unlistenError()
}

// Lifecycle hooks
onMounted(() => {
  setupEventListeners()
})

onUnmounted(() => {
  cleanupEventListeners()
})
</script>

<template>
  <div class="ai-chat-panel" :style="{ width: panelWidth }">
    <!-- Header -->
    <div class="panel-header">
      <template v-if="!isCollapsed">
        <div class="header-content">
          <h3 class="panel-title">AI Assistant</h3>
          <span class="provider-badge">{{ providerBadge }}</span>
        </div>
        <button class="collapse-btn" @click="toggleCollapse" title="Collapse panel">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </button>
      </template>
      <button v-else class="expand-btn" @click="toggleCollapse" title="Expand panel">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M15 18l-6-6 6-6" />
        </svg>
      </button>
    </div>

    <!-- Messages Area -->
    <div v-if="!isCollapsed" class="messages-container" ref="messagesContainer">
      <!-- Empty state -->
      <div v-if="!aiStore.hasMessages && !aiStore.isStreaming" class="empty-state">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2z" />
            <path d="M12 16v-4" />
            <path d="M12 8h.01" />
          </svg>
        </div>
        <p class="empty-text">Ask me anything about your terminal commands</p>
        <p class="empty-hint">Type a question below to get started</p>
      </div>

      <!-- Messages list -->
      <div v-for="message in aiStore.messages" :key="message.id" class="message" :class="message.role">
        <div class="message-header">
          <span class="message-role">{{ message.role === 'user' ? 'You' : 'AI' }}</span>
          <span class="message-time">{{ formatTime(message.timestamp) }}</span>
        </div>
        <div class="message-content" v-html="renderMarkdown(message.content)"></div>

        <!-- Command buttons -->
        <div v-if="message.role === 'assistant'" class="command-buttons">
          <button
            v-for="(cmd, idx) in extractCommands(message.content)"
            :key="idx"
            class="cmd-btn"
            @click="executeCommand(cmd)"
            :title="`Execute: ${cmd}`"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M5 12h14M12 5l7 7-7 7" />
            </svg>
            <span class="cmd-text">{{ cmd }}</span>
          </button>
        </div>
      </div>

      <!-- Streaming message -->
      <div v-if="aiStore.isStreaming && aiStore.streamingContent" class="message assistant streaming">
        <div class="message-header">
          <span class="message-role">AI</span>
          <span class="streaming-indicator">
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
          </span>
        </div>
        <div class="message-content" v-html="renderMarkdown(aiStore.streamingContent)"></div>
      </div>

      <!-- Loading indicator -->
      <div v-if="aiStore.isStreaming && !aiStore.streamingContent" class="message assistant loading">
        <div class="loading-indicator">
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>
      </div>
    </div>

    <!-- Input Area -->
    <div v-if="!isCollapsed" class="input-area">
      <textarea
        v-model="inputText"
        class="input-textarea"
        placeholder="Ask about commands..."
        :disabled="aiStore.isStreaming"
        @keydown="handleKeydown"
        rows="2"
      ></textarea>
      <button
        class="send-btn"
        :disabled="!inputText.trim() || aiStore.isStreaming"
        @click="sendMessage"
        title="Send message (Enter)"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.ai-chat-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #181825;
  border-left: 1px solid #313244;
  transition: width 0.2s ease;
  overflow: hidden;
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background-color: #1e1e2e;
  border-bottom: 1px solid #313244;
  min-height: 48px;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #cdd6f4;
}

.provider-badge {
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 500;
  color: #89b4fa;
  background-color: #313244;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.collapse-btn,
.expand-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: transparent;
  border: none;
  color: #6c7086;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.collapse-btn:hover,
.expand-btn:hover {
  background-color: #313244;
  color: #cdd6f4;
}

.expand-btn {
  margin: 0 auto;
}

/* Messages Container */
.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  scrollbar-width: thin;
  scrollbar-color: #45475a #181825;
}

.messages-container::-webkit-scrollbar {
  width: 6px;
}

.messages-container::-webkit-scrollbar-track {
  background: #181825;
}

.messages-container::-webkit-scrollbar-thumb {
  background-color: #45475a;
  border-radius: 3px;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding: 24px;
}

.empty-icon {
  color: #45475a;
  margin-bottom: 16px;
}

.empty-text {
  margin: 0 0 8px;
  font-size: 14px;
  color: #a6adc8;
}

.empty-hint {
  margin: 0;
  font-size: 12px;
  color: #6c7086;
}

/* Messages */
.message {
  margin-bottom: 16px;
  padding: 12px;
  border-radius: 8px;
}

.message.user {
  background-color: #313244;
}

.message.assistant {
  background-color: #11111b;
}

.message.system {
  background-color: #45475a;
  color: #f38ba8;
}

.message-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.message-role {
  font-size: 11px;
  font-weight: 600;
  color: #89b4fa;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.message.system .message-role {
  color: #f38ba8;
}

.message-time {
  font-size: 10px;
  color: #6c7086;
}

.message-content {
  font-size: 13px;
  line-height: 1.5;
  color: #cdd6f4;
  word-wrap: break-word;
}

/* Code Blocks */
.message-content :deep(.code-block) {
  margin: 8px 0;
  border-radius: 6px;
  overflow: hidden;
  background-color: #11111b;
  border: 1px solid #313244;
}

.message-content :deep(.code-header) {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  background-color: #181825;
  border-bottom: 1px solid #313244;
}

.message-content :deep(.code-header span) {
  font-size: 11px;
  color: #6c7086;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.message-content :deep(pre) {
  margin: 0;
  padding: 12px;
  overflow-x: auto;
  scrollbar-width: thin;
  scrollbar-color: #45475a #11111b;
}

.message-content :deep(code) {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
  color: #a6adc8;
}

.message-content :deep(.inline-code) {
  padding: 2px 6px;
  background-color: #1e1e2e;
  border-radius: 3px;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #f9e2af;
}

/* Command Buttons */
.command-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #313244;
}

.cmd-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background-color: #1e1e2e;
  border: 1px solid #45475a;
  border-radius: 4px;
  color: #a6e3a1;
  font-size: 11px;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cmd-btn:hover {
  background-color: #313244;
  border-color: #a6e3a1;
}

.cmd-text {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Streaming & Loading */
.streaming-indicator,
.loading-indicator {
  display: flex;
  gap: 4px;
  align-items: center;
}

.streaming-indicator .dot,
.loading-indicator .dot {
  width: 6px;
  height: 6px;
  background-color: #89b4fa;
  border-radius: 50%;
  animation: pulse 1.4s infinite ease-in-out both;
}

.streaming-indicator .dot:nth-child(1),
.loading-indicator .dot:nth-child(1) {
  animation-delay: -0.32s;
}

.streaming-indicator .dot:nth-child(2),
.loading-indicator .dot:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes pulse {
  0%, 80%, 100% {
    opacity: 0.4;
    transform: scale(0.8);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}

.message.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

/* Input Area */
.input-area {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  background-color: #1e1e2e;
  border-top: 1px solid #313244;
}

.input-textarea {
  flex: 1;
  padding: 8px 12px;
  background-color: #181825;
  border: 1px solid #45475a;
  border-radius: 6px;
  color: #cdd6f4;
  font-size: 13px;
  font-family: inherit;
  line-height: 1.4;
  resize: none;
  outline: none;
  transition: border-color 0.15s ease;
}

.input-textarea::placeholder {
  color: #6c7086;
}

.input-textarea:focus {
  border-color: #89b4fa;
}

.input-textarea:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  background-color: #89b4fa;
  border: none;
  border-radius: 6px;
  color: #1e1e2e;
  cursor: pointer;
  transition: all 0.15s ease;
  align-self: flex-end;
}

.send-btn:hover:not(:disabled) {
  background-color: #b4befe;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
