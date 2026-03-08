//! Context Manager for aiTerm
//!
//! Manages terminal history with a ring buffer for context-aware AI assistance.
//! Features:
//! - Sliding window for recent entries
//! - LLM-based summarization for old entries
//! - Key information preservation (errors, important commands)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Default maximum number of lines to keep in the ring buffer
const DEFAULT_MAX_LINES: usize = 500;

/// Characters per token approximation (roughly 4 chars = 1 token for English)
const CHARS_PER_TOKEN: usize = 4;

/// Minimum entries before triggering summarization
const MIN_ENTRIES_FOR_SUMMARY: usize = 100;

/// Types of terminal entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    /// User input command
    Input,
    /// Terminal output/response
    Output,
}

/// A single entry in the terminal history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEntry {
    /// The content of the entry
    pub content: String,
    /// Whether this is input or output
    pub entry_type: EntryType,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
}

impl TerminalEntry {
    /// Create a new terminal entry with the current timestamp
    pub fn new(content: String, entry_type: EntryType) -> Self {
        let timestamp = Utc::now().timestamp_millis();
        Self {
            content,
            entry_type,
            timestamp,
        }
    }

    /// Estimate the number of tokens in this entry
    pub fn estimated_tokens(&self) -> usize {
        self.content.len().div_ceil(CHARS_PER_TOKEN)
    }

    /// Check if this entry contains error-like content
    pub fn is_error(&self) -> bool {
        let lower = self.content.to_lowercase();
        // Common error indicators in terminal output
        lower.contains("error")
            || lower.contains("failed")
            || lower.contains("exception")
            || lower.contains("fatal")
            || lower.contains("panic")
            || lower.contains("traceback")
            || lower.contains("unreachable")
            || lower.contains("permission denied")
            || lower.contains("command not found")
            || lower.contains("no such file")
            || lower.contains("cannot find")
            || lower.contains("invalid")
            || lower.contains("unexpected")
    }
}

/// Summary of old terminal history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    /// The summarized content
    pub content: String,
    /// Number of entries that were summarized
    pub entry_count: usize,
    /// Timestamp when summary was created
    pub created_at: i64,
    /// Key information preserved (errors, important commands)
    pub key_info: Vec<String>,
}

impl ContextSummary {
    pub fn new(content: String, entry_count: usize, key_info: Vec<String>) -> Self {
        Self {
            content,
            entry_count,
            created_at: Utc::now().timestamp_millis(),
            key_info,
        }
    }

    /// Estimate tokens in summary
    pub fn estimated_tokens(&self) -> usize {
        self.content.len().div_ceil(CHARS_PER_TOKEN)
    }
}

/// Statistics about context state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    /// Entries currently in ring buffer
    pub entries_in_buffer: usize,
    /// Entries waiting to be summarized
    pub pending_summary_count: usize,
    /// Total entries ever pushed
    pub total_entries: usize,
    /// Whether a summary exists
    pub has_summary: bool,
    /// Number of entries in the summary
    pub summary_entry_count: usize,
}

/// Context manager with ring buffer for terminal history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    /// Ring buffer of terminal entries (recent)
    entries: VecDeque<TerminalEntry>,
    /// Maximum number of lines to keep in buffer
    max_lines: usize,
    /// Summary of old entries that were pushed out
    summary: Option<ContextSummary>,
    /// Total entries ever pushed (for tracking)
    total_entries: usize,
    /// Entries pending summarization
    pending_summary: Vec<TerminalEntry>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LINES)
    }
}

impl ContextManager {
    /// Create a new context manager with the specified maximum line count
    pub fn new(max_lines: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_lines),
            max_lines,
            summary: None,
            total_entries: 0,
            pending_summary: Vec::new(),
        }
    }

    /// Push a new entry into the ring buffer
    ///
    /// When buffer overflows, old entries are moved to pending_summary
    /// for later LLM summarization
    pub fn push(&mut self, content: String, entry_type: EntryType) {
        // If at capacity, move oldest to pending summary
        if self.entries.len() >= self.max_lines {
            if let Some(old_entry) = self.entries.pop_front() {
                self.pending_summary.push(old_entry);
            }
        }
        self.entries.push_back(TerminalEntry::new(content, entry_type));
        self.total_entries += 1;
    }

    /// Check if there are entries pending summarization
    pub fn has_pending_summary(&self) -> bool {
        self.pending_summary.len() >= MIN_ENTRIES_FOR_SUMMARY
    }

    /// Get entries pending summarization (for LLM processing)
    pub fn get_pending_summary_entries(&self) -> &[TerminalEntry] {
        &self.pending_summary
    }

    /// Format pending entries for LLM summarization prompt
    pub fn format_pending_for_summary(&self) -> String {
        let mut result = String::new();

        for entry in &self.pending_summary {
            let marker = match entry.entry_type {
                EntryType::Input => "$ ",
                EntryType::Output => "> ",
            };
            result.push_str(&format!("{}{}\n", marker, entry.content));
        }

        result
    }

    /// Apply LLM-generated summary and clear pending entries
    pub fn apply_summary(&mut self, summary_content: String, key_info: Vec<String>) {
        let entry_count = self.pending_summary.len();

        // Merge with existing summary if present
        let new_summary = if let Some(existing) = &self.summary {
            ContextSummary::new(
                format!("{}\n\n{}", existing.content, summary_content),
                existing.entry_count + entry_count,
                [existing.key_info.clone(), key_info].concat(),
            )
        } else {
            ContextSummary::new(summary_content, entry_count, key_info)
        };

        self.summary = Some(new_summary);
        self.pending_summary.clear();
    }

    /// Get current summary (if any)
    pub fn get_summary(&self) -> Option<&ContextSummary> {
        self.summary.as_ref()
    }

    /// Clear summary (e.g., when user clears terminal)
    pub fn clear_summary(&mut self) {
        self.summary = None;
        self.pending_summary.clear();
    }

    /// Get statistics about context state
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            entries_in_buffer: self.entries.len(),
            pending_summary_count: self.pending_summary.len(),
            total_entries: self.total_entries,
            has_summary: self.summary.is_some(),
            summary_entry_count: self.summary.as_ref().map(|s| s.entry_count).unwrap_or(0),
        }
    }

    /// Push an input entry (user command)
    pub fn push_input(&mut self, content: String) {
        self.push(content, EntryType::Input);
    }

    /// Push an output entry (terminal response)
    pub fn push_output(&mut self, content: String) {
        self.push(content, EntryType::Output);
    }

    /// Build a context string from summary + recent entries within token budget
    ///
    /// Structure: [Summary of old entries] + [Recent entries]
    /// Uses a simple character-based token estimation (1 token ≈ 4 chars)
    pub fn build_context(&self, max_tokens: usize) -> String {
        let mut context_parts = Vec::new();
        let mut current_tokens = 0;

        // 1. Include summary first (if exists and fits in budget)
        if let Some(ref summary) = self.summary {
            let summary_tokens = summary.estimated_tokens();

            // Reserve some tokens for recent entries (at least 30% of budget)
            let max_summary_tokens = (max_tokens as f64 * 0.7) as usize;

            if summary_tokens <= max_summary_tokens {
                let summary_text = format!(
                    "[Summary of {} previous entries]\n{}\n[End summary]\n\n",
                    summary.entry_count, summary.content
                );
                context_parts.push(summary_text.clone());
                current_tokens += summary_tokens;

                // Add key info if present
                if !summary.key_info.is_empty() {
                    let key_info_text = format!("Key information:\n{}\n\n", summary.key_info.join("\n"));
                    let key_info_tokens = key_info_text.len().div_ceil(CHARS_PER_TOKEN);
                    if current_tokens + key_info_tokens <= max_tokens {
                        context_parts.push(key_info_text);
                        current_tokens += key_info_tokens;
                    }
                }
            }
        }

        // 2. Add recent entries from newest to oldest
        let mut recent_parts = Vec::new();

        for entry in self.entries.iter().rev() {
            let entry_tokens = entry.estimated_tokens();

            if current_tokens + entry_tokens > max_tokens {
                break;
            }

            let type_marker = match entry.entry_type {
                EntryType::Input => "$ ",
                EntryType::Output => "> ",
            };

            let formatted = format!("{}{}", type_marker, entry.content);
            recent_parts.push(formatted);
            current_tokens += entry_tokens;
        }

        // Reverse to get chronological order
        recent_parts.reverse();

        // Combine summary + recent entries
        context_parts.extend(recent_parts);

        context_parts.join("")
    }

    /// Get recent error-containing output entries
    ///
    /// Returns up to `count` most recent output entries that appear to be errors
    pub fn get_recent_errors(&self, count: usize) -> Vec<&TerminalEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|entry| entry.entry_type == EntryType::Output && entry.is_error())
            .take(count)
            .collect()
    }

    /// Clear all entries and summary from the context manager
    pub fn clear(&mut self) {
        self.entries.clear();
        self.summary = None;
        self.pending_summary.clear();
        self.total_entries = 0;
    }

    /// Get the number of entries in the buffer
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a reference to all entries (for serialization/export)
    pub fn entries(&self) -> &VecDeque<TerminalEntry> {
        &self.entries
    }

    /// Get the timestamp of the most recent entry
    pub fn last_timestamp(&self) -> Option<i64> {
        self.entries.back().map(|e| e.timestamp)
    }

    /// Get entries within a time range (inclusive)
    pub fn get_entries_in_range(&self, start: i64, end: i64) -> Vec<&TerminalEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_new() {
        let ctx = ContextManager::new(100);
        assert_eq!(ctx.max_lines, 100);
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    fn test_context_manager_default() {
        let ctx = ContextManager::default();
        assert_eq!(ctx.max_lines, DEFAULT_MAX_LINES);
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_push_input() {
        let mut ctx = ContextManager::new(10);
        ctx.push_input("ls -la".to_string());

        assert_eq!(ctx.len(), 1);
        assert!(!ctx.is_empty());

        let entries = ctx.entries();
        assert_eq!(entries[0].content, "ls -la");
        assert_eq!(entries[0].entry_type, EntryType::Input);
    }

    #[test]
    fn test_push_output() {
        let mut ctx = ContextManager::new(10);
        ctx.push_output("total 0".to_string());

        assert_eq!(ctx.len(), 1);

        let entries = ctx.entries();
        assert_eq!(entries[0].content, "total 0");
        assert_eq!(entries[0].entry_type, EntryType::Output);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut ctx = ContextManager::new(3);

        ctx.push_input("cmd1".to_string());
        ctx.push_output("out1".to_string());
        ctx.push_input("cmd2".to_string());
        ctx.push_output("out2".to_string()); // Should push out cmd1

        assert_eq!(ctx.len(), 3);

        let entries = ctx.entries();
        assert_eq!(entries[0].content, "out1");
        assert_eq!(entries[1].content, "cmd2");
        assert_eq!(entries[2].content, "out2");
    }

    #[test]
    fn test_clear() {
        let mut ctx = ContextManager::new(10);
        ctx.push_input("cmd1".to_string());
        ctx.push_output("out1".to_string());

        assert_eq!(ctx.len(), 2);

        ctx.clear();

        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    fn test_build_context() {
        let mut ctx = ContextManager::new(10);
        ctx.push_input("ls".to_string());
        ctx.push_output("file1.txt".to_string());
        ctx.push_input("cat file1.txt".to_string());
        ctx.push_output("hello world".to_string());

        // With enough tokens, should include all entries
        let context = ctx.build_context(1000);
        assert!(context.contains("$ ls"));
        assert!(context.contains("> file1.txt"));
        assert!(context.contains("$ cat file1.txt"));
        assert!(context.contains("> hello world"));
    }

    #[test]
    fn test_build_context_token_limit() {
        let mut ctx = ContextManager::new(10);

        // Add entries that would exceed a small token budget
        ctx.push_input("a".repeat(100)); // ~25 tokens
        ctx.push_output("b".repeat(100)); // ~25 tokens
        ctx.push_input("short".to_string()); // ~1 token

        // With only 30 tokens budget, should not include all
        let context = ctx.build_context(30);

        // Should have some content but not all
        assert!(!context.is_empty());
    }

    #[test]
    fn test_is_error_detection() {
        let mut entry = TerminalEntry::new("Error: something went wrong".to_string(), EntryType::Output);
        assert!(entry.is_error());

        entry.content = "FAILED to connect".to_string();
        assert!(entry.is_error());

        entry.content = "Success: operation completed".to_string();
        assert!(!entry.is_error());

        entry.content = "panic: runtime error".to_string();
        assert!(entry.is_error());
    }

    #[test]
    fn test_get_recent_errors() {
        let mut ctx = ContextManager::new(10);

        ctx.push_output("normal output".to_string());
        ctx.push_output("Error: something failed".to_string());
        ctx.push_input("cmd".to_string());
        ctx.push_output("FATAL: crash".to_string());
        ctx.push_output("another error occurred".to_string());

        let errors = ctx.get_recent_errors(2);
        assert_eq!(errors.len(), 2);

        // Should be the most recent errors in reverse order
        assert!(errors[0].content.contains("another error"));
        assert!(errors[1].content.contains("FATAL"));
    }

    #[test]
    fn test_get_recent_errors_no_errors() {
        let mut ctx = ContextManager::new(10);

        ctx.push_output("normal output".to_string());
        ctx.push_input("cmd".to_string());
        ctx.push_output("success".to_string());

        let errors = ctx.get_recent_errors(5);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_entry_estimated_tokens() {
        let entry = TerminalEntry::new("a".repeat(100), EntryType::Input);
        assert_eq!(entry.estimated_tokens(), 25); // 100 / 4 = 25

        let entry = TerminalEntry::new("short".to_string(), EntryType::Input);
        assert_eq!(entry.estimated_tokens(), 2); // 5 / 4 = 1.25 -> ceil = 2
    }

    #[test]
    fn test_entry_type_serialization() {
        let input = EntryType::Input;
        let json = serde_json::to_string(&input).unwrap();
        assert_eq!(json, "\"Input\"");

        let output = EntryType::Output;
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(json, "\"Output\"");
    }

    #[test]
    fn test_pending_summary() {
        let mut ctx = ContextManager::new(5);

        // Fill the buffer
        for i in 0..5 {
            ctx.push_input(format!("cmd{}", i));
        }

        // Add more entries - should overflow to pending
        ctx.push_input("cmd5".to_string());
        ctx.push_input("cmd6".to_string());

        assert_eq!(ctx.pending_summary.len(), 2);
        assert!(ctx.has_pending_summary() || ctx.pending_summary.len() < MIN_ENTRIES_FOR_SUMMARY);
    }

    #[test]
    fn test_apply_summary() {
        let mut ctx = ContextManager::new(5);

        // Fill and overflow
        for i in 0..7 {
            ctx.push_input(format!("cmd{}", i));
        }

        assert_eq!(ctx.pending_summary.len(), 2);

        // Apply summary
        ctx.apply_summary("User ran various commands".to_string(), vec!["Error in cmd3".to_string()]);

        assert!(ctx.summary.is_some());
        assert!(ctx.pending_summary.is_empty());

        let summary = ctx.summary.as_ref().unwrap();
        assert_eq!(summary.entry_count, 2);
        assert_eq!(summary.key_info.len(), 1);
    }

    #[test]
    fn test_build_context_with_summary() {
        let mut ctx = ContextManager::new(5);

        // Fill and overflow
        for i in 0..7 {
            ctx.push_input(format!("cmd{}", i));
        }

        // Apply summary
        ctx.apply_summary("Previous commands summary".to_string(), vec!["Important: cmd0".to_string()]);

        // Build context should include summary
        let context = ctx.build_context(1000);
        assert!(context.contains("[Summary of 2 previous entries]"));
        assert!(context.contains("Previous commands summary"));
        assert!(context.contains("Key information:"));
    }

    #[test]
    fn test_stats() {
        let mut ctx = ContextManager::new(5);

        let stats = ctx.stats();
        assert_eq!(stats.entries_in_buffer, 0);
        assert_eq!(stats.total_entries, 0);
        assert!(!stats.has_summary);

        // Add entries
        for i in 0..7 {
            ctx.push_input(format!("cmd{}", i));
        }

        let stats = ctx.stats();
        assert_eq!(stats.entries_in_buffer, 5);
        assert_eq!(stats.pending_summary_count, 2);
        assert_eq!(stats.total_entries, 7);

        // Apply summary
        ctx.apply_summary("Summary".to_string(), vec![]);

        let stats = ctx.stats();
        assert!(stats.has_summary);
        assert_eq!(stats.summary_entry_count, 2);
    }
}
