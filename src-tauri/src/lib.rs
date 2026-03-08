//! aiTerm - AI-First Local Terminal
//!
//! This is the main entry point for the Tauri application backend.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, State};

pub mod config;
pub mod context;
pub mod llm;
pub mod pty;

// Re-export PTY commands for convenience
pub use pty::{
    pty_create, pty_destroy, pty_exists, pty_resize, pty_session_count, pty_write, PtyManager,
};

// Re-export context types for commands
pub use context::{ContextManager, ContextStats, EntryType, TerminalEntry};

// Re-export config types
pub use config::AppConfig;

// Re-export LLM types
pub use llm::{ChatMessage, LlmClient, LlmProvider};

// ============================================================================
// Event Payloads
// ============================================================================

/// Payload for AI streaming events
#[derive(Clone, serde::Serialize)]
pub struct StreamPayload {
    pub content: String,
}

// ============================================================================
// Keyring API Key Management
// ============================================================================

const KEYRING_SERVICE: &str = "aiTerm";
const KEYRING_USERNAME: &str = "openai_api_key";

/// Get API key from system keyring
fn get_api_key() -> Result<String, String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .get_password()
        .map_err(|e| format!("Failed to get API key: {}", e))
}

/// Set API key in system keyring
fn set_api_key(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(key)
        .map_err(|e| format!("Failed to set API key: {}", e))
}

// ============================================================================
// Application State
// ============================================================================

/// Global application state shared across all commands
pub struct AppState {
    pub pty: Mutex<PtyManager>,
    pub config: Mutex<config::AppConfig>,
    pub context: Mutex<context::ContextManager>,
    pub llm: Mutex<Option<llm::LlmClient>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pty: Mutex::new(PtyManager::new()),
            config: Mutex::new(config::AppConfig::default()),
            context: Mutex::new(ContextManager::new(
                config::ContextConfig::default().max_lines,
            )),
            llm: Mutex::new(None),
        }
    }
}

// ============================================================================
// Settings Commands
// ============================================================================

/// Get current application settings
#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to lock config".to_string())?;

    Ok(config.clone())
}

/// Save application settings
#[tauri::command]
fn save_settings(settings: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    // Save to file
    settings.save().map_err(|e| e.to_string())?;

    // Update in-memory state
    let mut config = state
        .config
        .lock()
        .map_err(|_| "Failed to lock config".to_string())?;

    *config = settings;

    Ok(())
}

// ============================================================================
// AI Commands
// ============================================================================

/// Set the AI provider and optionally update API key
#[tauri::command]
fn ai_set_provider(
    provider: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save API key to keyring if provided
    if let Some(key) = &api_key {
        set_api_key(key)?;
    }

    // Get current config
    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to lock config".to_string())?;

    // Create new LLM client based on provider
    let llm_client = match provider.as_str() {
        "ollama" => {
            let ollama_config = &config.llm.ollama;
            LlmClient::new(LlmProvider::ollama_with_host(
                &ollama_config.host,
                &ollama_config.model,
            ))
        }
        "openai" => {
            let openai_config = &config.llm.openai;

            // Get API key from keyring or parameter
            let key = match api_key {
                Some(ref k) => k.clone(),
                None => get_api_key().map_err(|_| {
                    "No API key found. Please provide an API key.".to_string()
                })?,
            };

            LlmClient::new(LlmProvider::openai_compatible(
                &openai_config.api_base,
                key,
                &openai_config.model,
            ))
        }
        _ => {
            return Err(format!(
                "Unknown provider: {}. Supported: ollama, openai",
                provider
            ))
        }
    };

    // Update config provider
    drop(config); // Release lock before acquiring mutable

    let mut config = state
        .config
        .lock()
        .map_err(|_| "Failed to lock config".to_string())?;

    config.llm.provider = provider.clone();
    if api_key.is_some() {
        config.llm.openai.has_api_key = true;
    }

    // Update LLM client
    let mut llm = state
        .llm
        .lock()
        .map_err(|_| "Failed to lock LLM client".to_string())?;

    *llm = Some(llm_client);

    Ok(())
}

/// List available models from the current provider
#[tauri::command]
async fn ai_list_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let llm_client = {
        let llm_guard = state
            .llm
            .lock()
            .map_err(|_| "Failed to lock LLM client".to_string())?;

        llm_guard
            .as_ref()
            .ok_or_else(|| "No LLM client configured. Call ai_set_provider first.".to_string())?
            .clone()
    };

    llm_client.list_models().await.map_err(|e| e.to_string())
}

/// Test connection to the current provider
#[tauri::command]
async fn ai_test_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let llm_client = {
        let llm_guard = state
            .llm
            .lock()
            .map_err(|_| "Failed to lock LLM client".to_string())?;

        llm_guard
            .as_ref()
            .ok_or_else(|| "No LLM client configured. Call ai_set_provider first.".to_string())?
            .clone()
    };

    llm_client.test_connection().await.map_err(|e| e.to_string())
}

/// Send a chat message to the AI and stream the response
#[tauri::command]
async fn ai_chat(
    message: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Build context from context manager
    let context_str = {
        let context = state
            .context
            .lock()
            .map_err(|_| "Failed to lock context".to_string())?;

        let config = state
            .config
            .lock()
            .map_err(|_| "Failed to lock config".to_string())?;

        let ctx = context.build_context(config.context.max_tokens);
        ctx
    };

    // Build messages with system prompt and user message
    let system_prompt = llm::build_system_prompt();

    let mut messages = vec![ChatMessage::system(&system_prompt)];

    // Add context if available
    if !context_str.is_empty() {
        messages.push(ChatMessage::user(format!(
            "Current terminal context:\n{}",
            context_str
        )));
    }

    // Add the user's message
    messages.push(ChatMessage::user(&message));

    // Clone what we need for the async task
    let llm_guard = state
        .llm
        .lock()
        .map_err(|_| "Failed to lock LLM client".to_string())?;

    let llm_client = llm_guard
        .as_ref()
        .ok_or_else(|| "No LLM client configured. Call ai_set_provider first.".to_string())?
        .clone();

    drop(llm_guard);

    // Spawn tokio task to stream response
    tokio::spawn(async move {
        match llm_client.chat_stream(messages).await {
            Ok(mut stream) => {
                use futures::StreamExt;

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // Emit streaming chunk with proper payload structure
                            if let Err(e) = app_handle.emit("ai_stream", StreamPayload { content: chunk }) {
                                eprintln!("Failed to emit ai_stream event: {}", e);
                            }
                        }
                        Err(e) => {
                            let _ = app_handle.emit("ai_error", e.to_string());
                            break;
                        }
                    }
                }

                // Signal completion
                let _ = app_handle.emit("ai_done", ());
            }
            Err(e) => {
                let _ = app_handle.emit("ai_error", e.to_string());
            }
        }
    });

    Ok(())
}

// ============================================================================
// Context Commands
// ============================================================================

/// Push user input to context
#[tauri::command]
fn context_push_input(content: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    context.push_input(content);

    Ok(())
}

/// Push terminal output to context
#[tauri::command]
fn context_push_output(content: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    context.push_output(content);

    Ok(())
}

/// Get recent error entries from context
#[tauri::command]
fn context_get_errors(
    count: usize,
    state: State<'_, AppState>,
) -> Result<Vec<TerminalEntry>, String> {
    let context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    let errors: Vec<TerminalEntry> = context
        .get_recent_errors(count)
        .into_iter()
        .cloned()
        .collect();

    Ok(errors)
}

/// Clear all context entries
#[tauri::command]
fn context_clear(state: State<'_, AppState>) -> Result<(), String> {
    let mut context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    context.clear();

    Ok(())
}

/// Get context statistics
#[tauri::command]
fn context_stats(state: State<'_, AppState>) -> Result<ContextStats, String> {
    let context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    Ok(context.stats())
}

/// Check if there are entries pending summarization
#[tauri::command]
fn context_has_pending_summary(state: State<'_, AppState>) -> Result<bool, String> {
    let context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    Ok(context.has_pending_summary())
}

/// Get entries pending summarization (for LLM to summarize)
#[tauri::command]
fn context_get_pending_summary(state: State<'_, AppState>) -> Result<Vec<TerminalEntry>, String> {
    let context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    Ok(context.get_pending_summary_entries().to_vec())
}

/// Apply LLM-generated summary to context
#[tauri::command]
fn context_apply_summary(
    summary: String,
    key_info: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    context.apply_summary(summary, key_info);

    Ok(())
}

/// Get current context summary (if any)
#[tauri::command]
fn context_get_summary(state: State<'_, AppState>) -> Result<Option<context::ContextSummary>, String> {
    let context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    Ok(context.get_summary().cloned())
}

/// Summarize pending context entries using LLM
/// This is called automatically when there are enough pending entries
#[tauri::command]
async fn context_summarize(state: State<'_, AppState>) -> Result<(), String> {
    // Get pending entries and LLM client
    let (entries_text, llm_client) = {
        let context = state
            .context
            .lock()
            .map_err(|_| "Failed to lock context".to_string())?;

        let llm_guard = state
            .llm
            .lock()
            .map_err(|_| "Failed to lock LLM client".to_string())?;

        let llm_client = llm_guard
            .as_ref()
            .ok_or_else(|| "No LLM client configured".to_string())?
            .clone();

        let entries_text = context.format_pending_for_summary();

        (entries_text, llm_client)
    };

    // If no entries, nothing to do
    if entries_text.is_empty() {
        return Ok(());
    }

    // Build summarization messages
    let messages = llm::build_summarization_prompt(&entries_text);

    // Call LLM for summarization (non-streaming)
    let response = llm_client
        .chat(messages)
        .await
        .map_err(|e| format!("Summarization failed: {}", e))?;

    // Parse the summary result
    let summary_result = llm::SummaryResult::parse(&response);

    // Apply the summary
    let mut context = state
        .context
        .lock()
        .map_err(|_| "Failed to lock context".to_string())?;

    context.apply_summary(summary_result.summary, summary_result.key_info);

    Ok(())
}

// ============================================================================
// Legacy Commands
// ============================================================================

/// Legacy greet command (can be removed in production)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ============================================================================
// Application Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load config from file or use default
    let app_config = config::AppConfig::load().unwrap_or_default();

    // Initialize state
    let state = AppState {
        pty: Mutex::new(PtyManager::new()),
        config: Mutex::new(app_config.clone()),
        context: Mutex::new(ContextManager::new(app_config.context.max_lines)),
        llm: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Legacy
            greet,
            // PTY commands
            pty_create,
            pty_write,
            pty_resize,
            pty_destroy,
            pty_exists,
            pty_session_count,
            // Settings commands
            get_settings,
            save_settings,
            // AI commands
            ai_set_provider,
            ai_list_models,
            ai_test_connection,
            ai_chat,
            // Context commands
            context_push_input,
            context_push_output,
            context_get_errors,
            context_clear,
            context_stats,
            context_has_pending_summary,
            context_get_pending_summary,
            context_apply_summary,
            context_get_summary,
            context_summarize,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
