import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AIMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: Date
}

export interface AIConversation {
  id: string
  messages: AIMessage[]
  createdAt: Date
  updatedAt: Date
}

export const useAIStore = defineStore('ai', () => {
  const messages = ref<AIMessage[]>([])
  const isStreaming = ref(false)
  const streamingContent = ref('')
  const provider = ref<'ollama' | 'openai'>('ollama')
  const currentConversationId = ref<string | null>(null)

  const hasMessages = computed(() => messages.value.length > 0)

  function generateId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  function addMessage(role: 'user' | 'assistant' | 'system', content: string): AIMessage {
    const message: AIMessage = {
      id: generateId(),
      role,
      content,
      timestamp: new Date()
    }
    messages.value.push(message)
    return message
  }

  function clearMessages(): void {
    messages.value = []
  }

  function setProvider(newProvider: 'ollama' | 'openai'): void {
    provider.value = newProvider
  }

  function startStreaming(): void {
    isStreaming.value = true
    streamingContent.value = ''
  }

  function appendStreamingContent(content: string): void {
    streamingContent.value += content
  }

  function finishStreaming(): void {
    if (streamingContent.value) {
      addMessage('assistant', streamingContent.value)
    }
    isStreaming.value = false
    streamingContent.value = ''
  }

  function showError(error: string): void {
    addMessage('system', `Error: ${error}`)
    isStreaming.value = false
    streamingContent.value = ''
  }

  async function sendMessage(content: string): Promise<void> {
    if (!content.trim()) return

    // Add user message
    addMessage('user', content)

    // Start streaming
    startStreaming()

    try {
      await invoke('ai_chat', { message: content })
    } catch (error) {
      showError(error instanceof Error ? error.message : String(error))
    }
  }

  return {
    messages,
    isStreaming,
    streamingContent,
    provider,
    currentConversationId,
    hasMessages,
    addMessage,
    clearMessages,
    setProvider,
    startStreaming,
    appendStreamingContent,
    finishStreaming,
    showError,
    sendMessage
  }
})
