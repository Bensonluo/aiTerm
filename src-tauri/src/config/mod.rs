use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to create config directory: {0}")]
    DirectoryCreation(#[source] std::io::Error),

    #[error("Failed to read config file: {0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[source] serde_json::Error),

    #[error("Failed to write config file: {0}")]
    Write(#[source] std::io::Error),

    #[error("Failed to serialize config: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("Config directory not found")]
    DirectoryNotFound,
}

/// Ollama LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// OpenAI LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalConfig {
    pub shell: String,
    pub font_size: u8,
    pub font_family: String,
    pub theme: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: if cfg!(target_os = "windows") {
                "cmd.exe".to_string()
            } else {
                "/bin/zsh".to_string()
            },
            font_size: 14,
            font_family: "Menlo".to_string(),
            theme: "default".to_string(),
        }
    }
}

/// Context configuration for LLM interactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextConfig {
    pub max_lines: usize,
    pub max_tokens: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_lines: 1000,
            max_tokens: 4096,
        }
    }
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            args: Vec::new(),
            env: None,
        }
    }
}

/// MCP configuration for managing server connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpConfig {
    pub servers: Vec<McpServerConfig>,
    pub max_iterations: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            max_iterations: 10,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub terminal: TerminalConfig,
    pub context: ContextConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            terminal: TerminalConfig::default(),
            context: ContextConfig::default(),
            mcp: McpConfig::default(),
        }
    }
}

impl AppConfig {
    /// Get the configuration directory path (~/.aiterm/)
    fn config_dir() -> Result<PathBuf, ConfigError> {
        dirs::config_dir()
            .map(|p| p.join("aiterm"))
            .ok_or(ConfigError::DirectoryNotFound)
    }

    /// Get the configuration file path (~/.aiterm/config.json)
    fn config_file_path() -> Result<PathBuf, ConfigError> {
        Self::config_dir().map(|p| p.join("config.json"))
    }

    /// Load configuration from file, or return default if file doesn't exist
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path).map_err(ConfigError::Read)?;

        let config: AppConfig =
            serde_json::from_str(&content).map_err(ConfigError::Parse)?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_file_path()?;

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(ConfigError::DirectoryCreation)?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(ConfigError::Serialize)?;

        fs::write(&config_path, content).map_err(ConfigError::Write)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.llm.provider, "ollama");
        assert_eq!(config.llm.ollama.host, "http://localhost:11434");
        assert_eq!(config.llm.ollama.model, "llama3.2");
        assert_eq!(config.terminal.font_size, 14);
        assert_eq!(config.context.max_lines, 1000);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_custom_config() {
        let mut config = AppConfig::default();
        config.llm.provider = "openai".to_string();
        config.llm.openai.has_api_key = true;
        config.terminal.font_size = 16;
        config.context.max_tokens = 8192;

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.llm.provider, "openai");
        assert!(deserialized.llm.openai.has_api_key);
        assert_eq!(deserialized.terminal.font_size, 16);
        assert_eq!(deserialized.context.max_tokens, 8192);
    }
}
