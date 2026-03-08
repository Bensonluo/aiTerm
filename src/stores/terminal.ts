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
