import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

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
  mcp: {
    servers: McpServerConfig[];
    maxIterations: number;
  };
}

export interface McpServerConfig {
  name: string;
  command: string;
  args: string[];
  argsText?: string;
  env?: Record<string, string>;
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
  mcp: {
    servers: [],
    maxIterations: 10,
  },
};

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({ ...defaultSettings });
  const isLoading = ref(false);

  const theme = computed(() => settings.value.terminal.theme);

  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    try {
      const loaded = await invoke<AppSettings>('get_settings');
      if (loaded) {
        settings.value = { ...defaultSettings, ...loaded };
      }
    } catch (error) {
      console.warn('Failed to load settings from backend, using defaults:', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings(): Promise<void> {
    try {
      await invoke('save_settings', { settings: settings.value });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  function updateTerminalTheme(newTheme: 'dark' | 'light'): void {
    settings.value.terminal.theme = newTheme;
  }

  return {
    settings,
    isLoading,
    theme,
    loadSettings,
    saveSettings,
    updateTerminalTheme,
  };
});
