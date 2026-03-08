//! LLM Client Module for aiTerm
//!
//! Provides unified interface for multiple LLM providers (Ollama and OpenAI-compatible APIs)

use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

use futures::StreamExt;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during LLM operations
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Connection test failed: {0}")]
    ConnectionError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

// ============================================================================
// Data Structures
// ============================================================================

/// Chat message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }
}

/// LLM Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmProvider {
    #[serde(rename = "ollama")]
    Ollama {
        #[serde(default = "default_ollama_host")]
        host: String,
        model: String,
    },

    #[serde(rename = "openai")]
    OpenAi {
        #[serde(default = "default_openai_base")]
        api_base: String,
        model: String,
        api_key: String,
    },
}

fn default_ollama_host() -> String {
    "http://localhost:11434".to_string()
}

fn default_openai_base() -> String {
    "https://api.openai.com/v1".to_string()
}

impl LlmProvider {
    /// Create a new Ollama provider with default host
    pub fn ollama(model: impl Into<String>) -> Self {
        Self::Ollama {
            host: default_ollama_host(),
            model: model.into(),
        }
    }

    /// Create a new Ollama provider with custom host
    pub fn ollama_with_host(host: impl Into<String>, model: impl Into<String>) -> Self {
        Self::Ollama {
            host: host.into(),
            model: model.into(),
        }
    }

    /// Create a new OpenAI provider
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::OpenAi {
            api_base: default_openai_base(),
            model: model.into(),
            api_key: api_key.into(),
        }
    }

    /// Create a new OpenAI-compatible provider with custom base URL
    pub fn openai_compatible(
        api_base: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self::OpenAi {
            api_base: api_base.into(),
            model: model.into(),
            api_key: api_key.into(),
        }
    }
}

// ============================================================================
// Ollama API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessage>,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

// ============================================================================
// OpenAI API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatChunk {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    delta: OpenAiDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    #[serde(default)]
    content: Option<String>,
}

// ============================================================================
// LLM Client
// ============================================================================

/// LLM Client that supports multiple providers
#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    provider: LlmProvider,
}

impl LlmClient {
    /// Create a new LLM client with the specified provider
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            client: Client::new(),
            provider,
        }
    }

    /// Set or change the provider
    pub fn set_provider(&mut self, provider: LlmProvider) {
        self.provider = provider;
    }

    /// Get the current provider
    pub fn provider(&self) -> &LlmProvider {
        &self.provider
    }

    /// List available models from the current provider
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, .. } => self.list_ollama_models(host).await,
            LlmProvider::OpenAi { api_base, api_key, .. } => {
                self.list_openai_models(api_base, api_key).await
            }
        }
    }

    /// Test connection to the current provider
    pub async fn test_connection(&self) -> Result<bool, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, .. } => {
                let url = format!("{}/api/tags", host);
                let response = self
                    .client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| LlmError::ConnectionError(e.to_string()))?;

                Ok(response.status().is_success())
            }
            LlmProvider::OpenAi { api_base, api_key, .. } => {
                let url = format!("{}/models", api_base);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(api_key)
                    .send()
                    .await
                    .map_err(|e| LlmError::ConnectionError(e.to_string()))?;

                Ok(response.status().is_success())
            }
        }
    }

    /// Send a chat request and return a stream of responses
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, model } => {
                self.ollama_chat_stream(host, model, messages).await
            }
            LlmProvider::OpenAi { api_base, model, api_key } => {
                self.openai_chat_stream(api_base, api_key, model, messages).await
            }
        }
    }

    /// Send a chat request and return the complete response (non-streaming)
    /// Used for summarization and other tasks that don't need streaming
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, LlmError> {
        match &self.provider {
            LlmProvider::Ollama { host, model } => {
                self.ollama_chat(host, model, messages).await
            }
            LlmProvider::OpenAi { api_base, model, api_key } => {
                self.openai_chat(api_base, api_key, model, messages).await
            }
        }
    }

    // ========================================================================
    // Ollama Implementation
    // ========================================================================

    async fn list_ollama_models(&self, host: &str) -> Result<Vec<String>, LlmError> {
        let url = format!("{}/api/tags", host);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(LlmError::ApiError(format!(
                "Failed to list models: {}",
                response.status()
            )));
        }

        let tags: OllamaTagsResponse = response.json().await?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    async fn ollama_chat_stream(
        &self,
        host: &str,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
        let url = format!("{}/api/chat", host);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages,
            stream: true,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Ollama request failed: {}",
                error_text
            )));
        }

        // Create a stream that parses each line as JSON
        let stream = async_stream::stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = byte_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<OllamaChatResponse>(&line) {
                                Ok(chunk) => {
                                    if let Some(message) = chunk.message {
                                        yield Ok(message.content);
                                    }
                                    if chunk.done {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    yield Err(LlmError::ParseError(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(LlmError::StreamError(e.to_string()));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    // ========================================================================
    // OpenAI Implementation
    // ========================================================================

    async fn list_openai_models(
        &self,
        api_base: &str,
        api_key: &str,
    ) -> Result<Vec<String>, LlmError> {
        let url = format!("{}/models", api_base);
        let response = self.client.get(&url).bearer_auth(api_key).send().await?;

        if !response.status().is_success() {
            return Err(LlmError::ApiError(format!(
                "Failed to list models: {}",
                response.status()
            )));
        }

        let models: OpenAiModelsResponse = response.json().await?;
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }

    async fn openai_chat_stream(
        &self,
        api_base: &str,
        api_key: &str,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
        let url = format!("{}/chat/completions", api_base);

        let request = OpenAiChatRequest {
            model: model.to_string(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "OpenAI request failed: {}",
                error_text
            )));
        }

        // Parse SSE format: "data: {json}" lines
        let stream = async_stream::stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = byte_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            // Skip comments and non-data lines
                            if !line.starts_with("data:") {
                                continue;
                            }

                            let data = line.strip_prefix("data:").unwrap_or("").trim();

                            // Check for end of stream
                            if data == "[DONE]" {
                                break;
                            }

                            match serde_json::from_str::<OpenAiChatChunk>(data) {
                                Ok(chunk) => {
                                    for choice in chunk.choices {
                                        if let Some(content) = choice.delta.content {
                                            yield Ok(content);
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(LlmError::ParseError(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(LlmError::StreamError(e.to_string()));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    // ========================================================================
    // Non-streaming implementations for summarization
    // ========================================================================

    async fn ollama_chat(
        &self,
        host: &str,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, LlmError> {
        let url = format!("{}/api/chat", host);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Ollama request failed: {}",
                error_text
            )));
        }

        let chat_response: OllamaChatResponse = response.json().await?;

        Ok(chat_response
            .message
            .map(|m| m.content)
            .unwrap_or_default())
    }

    async fn openai_chat(
        &self,
        api_base: &str,
        api_key: &str,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", api_base);

        #[derive(Debug, Serialize)]
        struct OpenAiChatRequestNonStreaming {
            model: String,
            messages: Vec<ChatMessage>,
        }

        let request = OpenAiChatRequestNonStreaming {
            model: model.to_string(),
            messages,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "OpenAI request failed: {}",
                error_text
            )));
        }

        #[derive(Debug, Deserialize)]
        struct OpenAiChatResponse {
            choices: Vec<OpenAiChatChoice>,
        }

        #[derive(Debug, Deserialize)]
        struct OpenAiChatChoice {
            message: OpenAiChatMessage,
        }

        #[derive(Debug, Deserialize)]
        struct OpenAiChatMessage {
            content: String,
        }

        let chat_response: OpenAiChatResponse = response.json().await?;

        Ok(chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

// ============================================================================
// System Prompt Builder
// ============================================================================

/// Build the system prompt for the AI terminal assistant
pub fn build_system_prompt() -> String {
    r#"你是一个专业的终端助手，帮助用户使用命令行界面。

你的职责：
1. 帮助用户理解和使用终端命令
2. 解释命令的用途和参数
3. 建议最佳实践和安全操作
4. 帮助调试命令行问题
5. 提供清晰、准确的中文回答

回答原则：
- 使用简洁明了的中文
- 提供具体的命令示例
- 解释命令的作用和注意事项
- 对于危险操作，明确警告用户
- 如果不确定，诚实告知并提供查询建议

当前环境信息可能包含在用户消息中，请结合上下文提供准确的帮助。"#
        .to_string()
}

/// Build the summarization prompt for context compression
pub fn build_summarization_prompt(entries_text: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::system(r#"你是一个终端历史总结助手。你的任务是将旧的终端历史压缩成简洁的摘要。

要求：
1. 保留重要信息：
   - 执行过的主要命令和操作
   - 遇到的错误和解决方法
   - 重要的配置更改
   - 关键的文件操作

2. 压缩方式：
   - 合并相似的命令
   - 省略冗长的输出细节
   - 保留错误信息和解决方案
   - 使用简洁的描述

3. 输出格式（JSON）：
{
  "summary": "简要描述用户做了什么操作",
  "key_info": ["重要的信息1", "重要的信息2"]
}

请严格按照JSON格式输出，不要添加其他内容。"#),
        ChatMessage::user(format!(
            "请总结以下终端历史：\n\n{}",
            entries_text
        )),
    ]
}

/// Summary result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    pub summary: String,
    pub key_info: Vec<String>,
}

impl SummaryResult {
    /// Parse from LLM response text
    pub fn parse(text: &str) -> Self {
        // Try to extract JSON from the response
        let text = text.trim();

        // Remove markdown code blocks if present
        let text = text.trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // Try to parse as JSON
        if let Ok(result) = serde_json::from_str::<Self>(text) {
            return result;
        }

        // Fallback: use the entire text as summary
        Self {
            summary: text.to_string(),
            key_info: Vec::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_new() {
        let msg = ChatMessage::new("user", "Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_chat_message_helpers() {
        let user_msg = ChatMessage::user("User content");
        assert_eq!(user_msg.role, "user");

        let assistant_msg = ChatMessage::assistant("Assistant content");
        assert_eq!(assistant_msg.role, "assistant");

        let system_msg = ChatMessage::system("System content");
        assert_eq!(system_msg.role, "system");
    }

    #[test]
    fn test_llm_provider_ollama() {
        let provider = LlmProvider::ollama("llama2");
        match provider {
            LlmProvider::Ollama { host, model } => {
                assert_eq!(host, "http://localhost:11434");
                assert_eq!(model, "llama2");
            }
            _ => panic!("Expected Ollama provider"),
        }
    }

    #[test]
    fn test_llm_provider_ollama_custom_host() {
        let provider = LlmProvider::ollama_with_host("http://custom:8080", "llama2");
        match provider {
            LlmProvider::Ollama { host, model } => {
                assert_eq!(host, "http://custom:8080");
                assert_eq!(model, "llama2");
            }
            _ => panic!("Expected Ollama provider"),
        }
    }

    #[test]
    fn test_llm_provider_openai() {
        let provider = LlmProvider::openai("sk-test", "gpt-4");
        match provider {
            LlmProvider::OpenAi { api_base, model, api_key } => {
                assert_eq!(api_base, "https://api.openai.com/v1");
                assert_eq!(model, "gpt-4");
                assert_eq!(api_key, "sk-test");
            }
            _ => panic!("Expected OpenAI provider"),
        }
    }

    #[test]
    fn test_llm_provider_openai_compatible() {
        let provider = LlmProvider::openai_compatible(
            "https://api.custom.com/v1",
            "custom-key",
            "custom-model",
        );
        match provider {
            LlmProvider::OpenAi { api_base, model, api_key } => {
                assert_eq!(api_base, "https://api.custom.com/v1");
                assert_eq!(model, "custom-model");
                assert_eq!(api_key, "custom-key");
            }
            _ => panic!("Expected OpenAI provider"),
        }
    }

    #[test]
    fn test_llm_client_new() {
        let provider = LlmProvider::ollama("llama2");
        let client = LlmClient::new(provider);

        match client.provider() {
            LlmProvider::Ollama { model, .. } => {
                assert_eq!(model, "llama2");
            }
            _ => panic!("Expected Ollama provider"),
        }
    }

    #[test]
    fn test_llm_client_set_provider() {
        let provider1 = LlmProvider::ollama("llama2");
        let provider2 = LlmProvider::openai("sk-test", "gpt-4");

        let mut client = LlmClient::new(provider1);
        client.set_provider(provider2);

        match client.provider() {
            LlmProvider::OpenAi { model, .. } => {
                assert_eq!(model, "gpt-4");
            }
            _ => panic!("Expected OpenAI provider"),
        }
    }

    #[test]
    fn test_build_system_prompt() {
        let prompt = build_system_prompt();
        assert!(prompt.contains("终端助手"));
        assert!(prompt.contains("命令"));
        assert!(prompt.len() > 100);
    }

    #[test]
    fn test_llm_error_display() {
        let error = LlmError::ConnectionError("Failed to connect".to_string());
        assert!(error.to_string().contains("Failed to connect"));

        let error = LlmError::ApiError("Rate limited".to_string());
        assert!(error.to_string().contains("Rate limited"));
    }

    #[test]
    fn test_provider_serialization() {
        let provider = LlmProvider::ollama("llama2");
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("ollama"));
        assert!(json.contains("llama2"));

        let provider = LlmProvider::openai("sk-test", "gpt-4");
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("openai"));
        assert!(json.contains("gpt-4"));
    }

    #[test]
    fn test_provider_deserialization() {
        let json = r#"{"type":"ollama","host":"http://localhost:11434","model":"llama2"}"#;
        let provider: LlmProvider = serde_json::from_str(json).unwrap();
        match provider {
            LlmProvider::Ollama { host, model } => {
                assert_eq!(host, "http://localhost:11434");
                assert_eq!(model, "llama2");
            }
            _ => panic!("Expected Ollama provider"),
        }

        let json = r#"{"type":"openai","api_base":"https://api.openai.com/v1","model":"gpt-4","api_key":"sk-test"}"#;
        let provider: LlmProvider = serde_json::from_str(json).unwrap();
        match provider {
            LlmProvider::OpenAi { api_base, model, api_key } => {
                assert_eq!(api_base, "https://api.openai.com/v1");
                assert_eq!(model, "gpt-4");
                assert_eq!(api_key, "sk-test");
            }
            _ => panic!("Expected OpenAI provider"),
        }
    }
}
