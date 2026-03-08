//! PTY Manager Module
//!
//! This module provides PTY (Pseudo-Terminal) management functionality
//! for spawning and interacting with shell processes.

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, Emitter, Runtime};
use thiserror::Error;

/// Payload for PTY output events
#[derive(Clone, Serialize)]
pub struct PtyOutputPayload {
    pub id: String,
    pub data: String,
}

/// Payload for PTY exit events
#[derive(Clone, Serialize)]
pub struct PtyExitPayload {
    pub id: String,
    pub exit_code: i32,
}

/// Errors that can occur during PTY operations
#[derive(Error, Debug)]
pub enum PtyError {
    #[error("Failed to create PTY: {0}")]
    CreationFailed(String),

    #[error("Failed to spawn shell: {0}")]
    SpawnFailed(String),

    #[error("PTY not found: {0}")]
    NotFound(String),

    #[error("Failed to write to PTY: {0}")]
    WriteFailed(String),

    #[error("Failed to resize PTY: {0}")]
    ResizeFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lock error")]
    LockError,
}

/// Result type for PTY operations
pub type PtyResult<T> = Result<T, PtyError>;

/// PTY session information
pub struct PtySession {
    /// The PTY pair (master + slave)
    pub pair: PtyPair,
    /// The writer for sending input to the shell
    pub writer: Box<dyn Write + Send>,
    /// The reader thread handle
    pub reader_handle: Option<thread::JoinHandle<()>>,
}

/// PTY Manager that handles multiple PTY sessions
pub struct PtyManager {
    /// Active PTY sessions indexed by ID
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    /// Create a new PTY manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new PTY session
    pub fn create<R: Runtime>(
        &self,
        app_handle: AppHandle<R>,
        id: String,
        cols: u16,
        rows: u16,
    ) -> PtyResult<()> {
        let pty_system = native_pty_system();

        // Create PTY pair with initial size
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreationFailed(e.to_string()))?;

        // Get shell command
        let shell = self.get_default_shell();
        let mut cmd = CommandBuilder::new(&shell);

        // Set up environment
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Spawn the shell
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // Get writer for input
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // Set up reader thread to emit output events
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let session_id = id.clone();
        let app_handle_clone = app_handle.clone();
        let sessions_clone = Arc::clone(&self.sessions);

        let reader_handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - process exited
                        let _ = app_handle_clone.emit("pty_exit", PtyExitPayload {
                            id: session_id.clone(),
                            exit_code: 0,
                        });
                        break;
                    }
                    Ok(n) => {
                        // Emit output event
                        if let Ok(output) = String::from_utf8(buffer[..n].to_vec()) {
                            let _ = app_handle_clone.emit("pty_output", PtyOutputPayload {
                                id: session_id.clone(),
                                data: output,
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("PTY read error: {}", e);
                        let _ = app_handle_clone.emit("pty_exit", PtyExitPayload {
                            id: session_id.clone(),
                            exit_code: 1,
                        });
                        break;
                    }
                }
            }

            // Clean up session
            if let Ok(mut sessions) = sessions_clone.lock() {
                sessions.remove(&session_id);
            }
        });

        // Store session
        let session = PtySession {
            pair,
            writer,
            reader_handle: Some(reader_handle),
        };

        let mut sessions = self.sessions.lock().map_err(|_| PtyError::LockError)?;
        sessions.insert(id.clone(), session);

        Ok(())
    }

    /// Write input to a PTY session
    pub fn write(&self, id: &str, data: &str) -> PtyResult<()> {
        let mut sessions = self.sessions.lock().map_err(|_| PtyError::LockError)?;

        let session = sessions
            .get_mut(id)
            .ok_or_else(|| PtyError::NotFound(id.to_string()))?;

        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| PtyError::WriteFailed(e.to_string()))?;

        session
            .writer
            .flush()
            .map_err(|e| PtyError::WriteFailed(e.to_string()))?;

        Ok(())
    }

    /// Resize a PTY session
    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> PtyResult<()> {
        let sessions = self.sessions.lock().map_err(|_| PtyError::LockError)?;

        let session = sessions
            .get(id)
            .ok_or_else(|| PtyError::NotFound(id.to_string()))?;

        session
            .pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::ResizeFailed(e.to_string()))?;

        Ok(())
    }

    /// Destroy a PTY session
    pub fn destroy(&self, id: &str) -> PtyResult<()> {
        let mut sessions = self.sessions.lock().map_err(|_| PtyError::LockError)?;

        if let Some(mut session) = sessions.remove(id) {
            // Drop the writer to signal EOF
            drop(session.writer);

            // Wait for reader thread to finish
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
        }

        Ok(())
    }

    /// Get the default shell for the current platform
    fn get_default_shell(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }

        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        }
    }

    /// Check if a session exists
    pub fn exists(&self, id: &str) -> bool {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.contains_key(id)
        } else {
            false
        }
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.len()
        } else {
            0
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

use std::sync::OnceLock;

static PTY_MANAGER: OnceLock<PtyManager> = OnceLock::new();

/// Get the global PTY manager instance
fn get_manager() -> &'static PtyManager {
    PTY_MANAGER.get_or_init(PtyManager::new)
}

/// Create a new PTY session
#[tauri::command]
pub fn pty_create<R: Runtime>(
    app_handle: AppHandle<R>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let manager = get_manager();
    manager.create(app_handle, id, cols, rows).map_err(|e| e.to_string())
}

/// Write data to a PTY session
#[tauri::command]
pub fn pty_write(id: String, data: String) -> Result<(), String> {
    let manager = get_manager();
    manager.write(&id, &data).map_err(|e| e.to_string())
}

/// Resize a PTY session
#[tauri::command]
pub fn pty_resize(id: String, cols: u16, rows: u16) -> Result<(), String> {
    let manager = get_manager();
    manager.resize(&id, cols, rows).map_err(|e| e.to_string())
}

/// Destroy a PTY session
#[tauri::command]
pub fn pty_destroy(id: String) -> Result<(), String> {
    let manager = get_manager();
    manager.destroy(&id).map_err(|e| e.to_string())
}

/// Check if a PTY session exists
#[tauri::command]
pub fn pty_exists(id: String) -> bool {
    let manager = get_manager();
    manager.exists(&id)
}

/// Get the number of active PTY sessions
#[tauri::command]
pub fn pty_session_count() -> usize {
    let manager = get_manager();
    manager.session_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_manager_creation() {
        let manager = PtyManager::new();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_default_shell() {
        let manager = PtyManager::new();
        let shell = manager.get_default_shell();
        assert!(!shell.is_empty());
    }
}
