<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

// Props
interface Props {
  visible: boolean
  command: string
  danger?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  danger: false
})

// Emits
const emit = defineEmits<{
  (e: 'confirm'): void
  (e: 'cancel'): void
  (e: 'edit', command: string): void
  (e: 'close'): void
}>()

// State
const isEditMode = ref(false)
const editedCommand = ref('')

// Dangerous command patterns - comprehensive detection
const dangerousPatterns = [
  // File deletion
  /\brm\s+(-[rf]+\s+|.*-.*[rf])/i,      // rm -rf, rm -r, rm -f
  /\bsudo\s+rm\b/i,                      // sudo rm
  /\brmdir\s+/i,                         // rmdir
  // Privilege escalation
  /\bsudo\s+/i,                          // sudo commands
  /\bsu\s+/i,                            // su command
  /\bdoas\s+/i,                          // doas command
  // Permission changes
  /\bchmod\s+-R\b/i,                     // chmod -R
  /\bchown\s+-R\b/i,                     // chown -R
  // Disk operations
  /\bdd\s+/i,                            // dd command
  /\bmkfs\b/i,                           // mkfs
  /\bfdisk\b/i,                          // fdisk
  /\bparted\b/i,                         // parted
  /\bgdisk\b/i,                          // gdisk
  // System power
  /\bshutdown\b/i,                       // shutdown
  /\breboot\b/i,                         // reboot
  /\bhalt\b/i,                           // halt
  /\bpoweroff\b/i,                       // poweroff
  /\binit\s+[06]\b/i,                    // init 0 or 6
  /\bsystemctl\s+(reboot|poweroff|halt)/i, // systemctl power commands
  // Fork bombs and dangerous constructs
  /:$$\)\{:\|:&\};:/,                    // fork bomb
  /\bfork\b.*\bbomb\b/i,                 // fork bomb reference
  // Device redirects
  />\s*\/dev\//i,                        // redirect to /dev/
  /\bmv\s+.*\s+\/dev\//i,               // move to /dev/
  /\b>\s*\/dev\/(sda|hda|nvme|vd|xvd)/i, // overwrite disk
  // Network operations
  /\biptables\b/i,                       // iptables
  /\bip6tables\b/i,                      // ip6tables
  /\bnft\b/i,                            // nftables
  // Command injection patterns
  /\$\(.*\)/,                            // command substitution $()
  /`.*`/,                                // backtick command substitution
  /\b(eval|exec)\s+/i,                   // eval/exec
  // Encoded/obfuscated commands
  /\\x[0-9a-f]{2}/i,                     // hex escapes
  /\\[0-7]{3}/,                          // octal escapes
  /\bbase64\s+-d\b/i,                    // base64 decode
  // Environment manipulation
  /\bexport\s+.*=\$?\(/i,                // export with command
  /\bsource\s+\/dev\//i,                 // source from /dev
  // Kill commands
  /\bkill\s+-9\s+1\b/i,                  // kill -9 1 (init)
  /\bkillall\s+/i,                       // killall
  /\bpkill\s+-9\b/i,                     // pkill -9
]

// Check if command is dangerous
const isDangerous = computed(() => {
  if (props.danger) return true
  return dangerousPatterns.some(pattern => pattern.test(props.command))
})

// Extract dangerous parts for highlighting
const highlightedCommand = computed(() => {
  if (!isDangerous.value) {
    return escapeHtml(props.command)
  }

  let result = escapeHtml(props.command)

  // Highlight dangerous patterns
  const highlightPatterns = [
    { pattern: /\b(rm\s+-rf?)/gi, class: 'danger-rm' },
    { pattern: /\b(sudo)\b/gi, class: 'danger-sudo' },
    { pattern: /\b(chmod|chown)\s+-R\b/gi, class: 'danger-perm' },
    { pattern: /\b(dd|mkfs|fdisk)\b/gi, class: 'danger-disk' },
    { pattern: /(>\s*\/dev\/\S*)/gi, class: 'danger-redirect' },
    { pattern: /\b(shutdown|reboot|halt)\b/gi, class: 'danger-power' },
  ]

  for (const { pattern, class: className } of highlightPatterns) {
    result = result.replace(pattern, `<span class="${className}">$1</span>`)
  }

  return result
})

// Escape HTML for safe display
function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

// Handle confirm
function handleConfirm(): void {
  if (isEditMode.value) {
    emit('edit', editedCommand.value)
  } else {
    emit('confirm')
  }
  closeModal()
}

// Handle cancel
function handleCancel(): void {
  emit('cancel')
  closeModal()
}

// Handle edit mode toggle
function toggleEditMode(): void {
  if (!isEditMode.value) {
    editedCommand.value = props.command
  }
  isEditMode.value = !isEditMode.value
}

// Handle edit confirm
function handleEditConfirm(): void {
  emit('edit', editedCommand.value)
  closeModal()
}

// Close modal
function closeModal(): void {
  isEditMode.value = false
  emit('close')
}

// Handle overlay click
function handleOverlayClick(event: MouseEvent): void {
  if (event.target === event.currentTarget) {
    handleCancel()
  }
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent): void {
  if (!props.visible) return

  // Ignore if typing in textarea
  if (event.target instanceof HTMLTextAreaElement) {
    if (event.key === 'Escape') {
      event.target.blur()
    }
    return
  }

  switch (event.key.toLowerCase()) {
    case 'y':
      event.preventDefault()
      handleConfirm()
      break
    case 'n':
      event.preventDefault()
      handleCancel()
      break
    case 'e':
      event.preventDefault()
      toggleEditMode()
      break
    case 'escape':
      event.preventDefault()
      handleCancel()
      break
  }
}

// Watch visibility to reset state
watch(() => props.visible, (newVal) => {
  if (newVal) {
    isEditMode.value = false
    editedCommand.value = props.command
  }
})

// Lifecycle hooks
onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Transition name="fade">
    <div
      v-if="visible"
      class="modal-overlay"
      @click="handleOverlayClick"
    >
      <Transition name="scale">
        <div v-if="visible" class="modal-content">
          <!-- Header -->
          <div class="modal-header">
            <div class="header-title">
              <span v-if="isDangerous" class="warning-icon">⚠️</span>
              <span v-if="isDangerous" class="title danger">Dangerous Command</span>
              <span v-else class="title">Confirm Command</span>
            </div>
            <button class="close-btn" @click="handleCancel" aria-label="Close">
              ✕
            </button>
          </div>

          <!-- Body -->
          <div class="modal-body">
            <p v-if="isDangerous" class="warning-text">
              This command may cause irreversible changes. Please review carefully before executing.
            </p>
            <p v-else class="info-text">
              Please confirm the following command:
            </p>

            <!-- Command Preview / Edit -->
            <div v-if="!isEditMode" class="command-box" :class="{ danger: isDangerous }">
              <code v-html="highlightedCommand"></code>
            </div>
            <div v-else class="edit-box">
              <textarea
                v-model="editedCommand"
                class="command-textarea"
                placeholder="Edit command..."
                rows="4"
                autofocus
              ></textarea>
            </div>
          </div>

          <!-- Footer -->
          <div class="modal-footer">
            <div class="keyboard-hints">
              <kbd>Y</kbd> Yes
              <kbd>N</kbd> No
              <kbd>E</kbd> Edit
              <kbd>Esc</kbd> Cancel
            </div>
            <div class="action-buttons">
              <button class="btn btn-edit" @click="toggleEditMode">
                {{ isEditMode ? 'Cancel Edit' : 'Edit' }}
              </button>
              <button class="btn btn-no" @click="handleCancel">
                No
              </button>
              <button
                v-if="isEditMode"
                class="btn btn-yes"
                @click="handleEditConfirm"
              >
                Execute Edited
              </button>
              <button
                v-else
                class="btn btn-yes"
                :class="{ danger: isDangerous }"
                @click="handleConfirm"
              >
                Yes, Execute
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
/* Overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* Modal Content */
.modal-content {
  background-color: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 12px;
  width: 90%;
  max-width: 560px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

/* Header */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #313244;
  background-color: rgba(49, 50, 68, 0.3);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.warning-icon {
  font-size: 20px;
}

.title {
  font-size: 16px;
  font-weight: 600;
  color: #cdd6f4;
}

.title.danger {
  color: #f38ba8;
}

.close-btn {
  background: none;
  border: none;
  color: #6c7086;
  font-size: 18px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #313244;
  color: #cdd6f4;
}

/* Body */
.modal-body {
  padding: 20px;
}

.warning-text {
  color: #f38ba8;
  font-size: 14px;
  margin-bottom: 16px;
  line-height: 1.5;
}

.info-text {
  color: #a6adc8;
  font-size: 14px;
  margin-bottom: 16px;
}

/* Command Box */
.command-box {
  background-color: #11111b;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
}

.command-box.danger {
  border-color: rgba(243, 139, 168, 0.3);
  background-color: rgba(243, 139, 168, 0.05);
}

.command-box code {
  font-family: 'JetBrains Mono', 'Fira Code', Menlo, Monaco, 'Courier New', monospace;
  font-size: 14px;
  color: #a6e3a1;
  white-space: pre-wrap;
  word-break: break-all;
}

/* Danger highlighting */
.command-box :deep(.danger-rm),
.command-box :deep(.danger-sudo),
.command-box :deep(.danger-perm),
.command-box :deep(.danger-disk),
.command-box :deep(.danger-redirect),
.command-box :deep(.danger-power) {
  color: #f38ba8;
  font-weight: 600;
  background-color: rgba(243, 139, 168, 0.15);
  padding: 2px 4px;
  border-radius: 3px;
}

/* Edit Box */
.edit-box {
  width: 100%;
}

.command-textarea {
  width: 100%;
  background-color: #11111b;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', Menlo, Monaco, 'Courier New', monospace;
  font-size: 14px;
  color: #a6e3a1;
  resize: vertical;
  min-height: 80px;
}

.command-textarea:focus {
  outline: none;
  border-color: #89b4fa;
}

.command-textarea::placeholder {
  color: #6c7086;
}

/* Footer */
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-top: 1px solid #313244;
  background-color: rgba(49, 50, 68, 0.2);
}

.keyboard-hints {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  color: #6c7086;
}

.keyboard-hints kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  padding: 0 6px;
  background-color: #313244;
  border: 1px solid #45475a;
  border-radius: 4px;
  font-family: inherit;
  font-size: 11px;
  color: #a6adc8;
}

.action-buttons {
  display: flex;
  gap: 10px;
}

/* Buttons */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-edit {
  background-color: #313244;
  color: #f9e2af;
}

.btn-edit:hover {
  background-color: #45475a;
}

.btn-no {
  background-color: #313244;
  color: #f38ba8;
}

.btn-no:hover {
  background-color: rgba(243, 139, 168, 0.2);
}

.btn-yes {
  background-color: #a6e3a1;
  color: #1e1e2e;
}

.btn-yes:hover {
  background-color: #94e2d5;
}

.btn-yes.danger {
  background-color: #f38ba8;
}

.btn-yes.danger:hover {
  background-color: #eba0ac;
}

/* Animations */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-enter-active,
.scale-leave-active {
  transition: all 0.2s ease;
}

.scale-enter-from,
.scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
