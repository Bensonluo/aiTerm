<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'

// Props
interface Props {
  fontSize?: number
  fontFamily?: string
  theme?: 'dark' | 'light'
}

const props = withDefaults(defineProps<Props>(), {
  fontSize: 14,
  fontFamily: 'Menlo, Monaco, "Courier New", monospace',
  theme: 'dark'
})

// Emits
const emit = defineEmits<{
  (e: 'ready'): void
  (e: 'data', data: string): void
  (e: 'resize', cols: number, rows: number): void
}>()

// Refs
const terminalContainer = ref<HTMLDivElement | null>(null)
let terminal: XTerm | null = null
let fitAddon: FitAddon | null = null

// Theme configurations
const themes = {
  dark: {
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    cursor: '#d4d4d4',
    cursorAccent: '#1e1e1e',
    selection: 'rgba(255, 255, 255, 0.3)',
    black: '#000000',
    red: '#cd3131',
    green: '#0dbc79',
    yellow: '#e5e510',
    blue: '#2472c8',
    magenta: '#bc3fbc',
    cyan: '#11a8cd',
    white: '#e5e5e5',
    brightBlack: '#666666',
    brightRed: '#f14c4c',
    brightGreen: '#23d18b',
    brightYellow: '#f5f543',
    brightBlue: '#3b8eea',
    brightMagenta: '#d670d6',
    brightCyan: '#29b8db',
    brightWhite: '#e5e5e5'
  },
  light: {
    background: '#ffffff',
    foreground: '#333333',
    cursor: '#333333',
    cursorAccent: '#ffffff',
    selection: 'rgba(0, 0, 0, 0.3)',
    black: '#000000',
    red: '#cd3131',
    green: '#00bc00',
    yellow: '#949800',
    blue: '#0451a5',
    magenta: '#bc05bc',
    cyan: '#0598bc',
    white: '#555555',
    brightBlack: '#666666',
    brightRed: '#cd3131',
    brightGreen: '#14ce14',
    brightYellow: '#b5ba00',
    brightBlue: '#0451a5',
    brightMagenta: '#bc05bc',
    brightCyan: '#0598bc',
    brightWhite: '#a5a5a5'
  }
}

// Initialize terminal
function initTerminal(): void {
  if (!terminalContainer.value) return

  // Create terminal instance
  terminal = new XTerm({
    fontSize: props.fontSize,
    fontFamily: props.fontFamily,
    theme: themes[props.theme],
    cursorBlink: true,
    cursorStyle: 'block',
    allowTransparency: true,
    scrollback: 10000,
    convertEol: true
  })

  // Create addons
  fitAddon = new FitAddon()
  const webLinksAddon = new WebLinksAddon()

  // Load addons
  terminal.loadAddon(fitAddon)
  terminal.loadAddon(webLinksAddon)

  // Open terminal in container
  terminal.open(terminalContainer.value)

  // Fit terminal to container
  fitAddon.fit()

  // Handle user input
  terminal.onData((data: string) => {
    emit('data', data)
  })

  // Handle resize
  terminal.onResize(({ cols, rows }) => {
    emit('resize', cols, rows)
  })

  // Emit ready event
  emit('ready')
}

// Handle window resize
function handleResize(): void {
  if (fitAddon && terminal) {
    fitAddon.fit()
  }
}

// Write data to terminal
function write(data: string): void {
  if (terminal) {
    terminal.write(data)
  }
}

// Write line to terminal
function writeln(data: string): void {
  if (terminal) {
    terminal.writeln(data)
  }
}

// Clear terminal
function clear(): void {
  if (terminal) {
    terminal.clear()
  }
}

// Focus terminal
function focus(): void {
  if (terminal) {
    terminal.focus()
  }
}

// Get terminal size
function getSize(): { cols: number; rows: number } | null {
  if (terminal) {
    return {
      cols: terminal.cols,
      rows: terminal.rows
    }
  }
  return null
}

// Expose methods to parent component
defineExpose({
  write,
  writeln,
  clear,
  focus,
  getSize
})

// Watch for theme changes
watch(
  () => props.theme,
  (newTheme) => {
    if (terminal) {
      terminal.options.theme = themes[newTheme]
    }
  }
)

// Watch for font size changes
watch(
  () => props.fontSize,
  (newSize) => {
    if (terminal) {
      terminal.options.fontSize = newSize
      fitAddon?.fit()
    }
  }
)

// Lifecycle hooks
onMounted(() => {
  initTerminal()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  if (terminal) {
    terminal.dispose()
    terminal = null
  }
})
</script>

<template>
  <div class="terminal-wrapper">
    <div ref="terminalContainer" class="terminal-container"></div>
  </div>
</template>

<style scoped>
.terminal-wrapper {
  width: 100%;
  height: 100%;
  background-color: v-bind('theme === "dark" ? "#1e1e1e" : "#ffffff"');
  overflow: hidden;
}

.terminal-container {
  width: 100%;
  height: 100%;
  padding: 8px;
  box-sizing: border-box;
}

/* Override xterm.js styles */
:deep(.xterm) {
  height: 100%;
}

:deep(.xterm-viewport) {
  overflow-y: auto;
}

:deep(.xterm-screen) {
  padding: 0;
}
</style>
