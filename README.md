# aiTerm - AI-First Local Terminal

An intelligent terminal application with built-in AI assistance, built with Tauri 2.0, Vue 3, and Rust.

## Features

- **Real Terminal Emulation** - Full PTY support with xterm.js rendering
- **AI Assistant** - Built-in AI chat to help with commands, debugging, and explanations
- **Context-Aware** - AI has full context of your terminal session for intelligent assistance
- **Smart Context Management** - Sliding window + LLM summarization for long sessions
- **Dual Provider Support** - Works with Ollama (local) or OpenAI-compatible APIs
- **Command Execution** - AI can suggest commands with one-click execution
- **Privacy First** - All terminal data stays local, API keys stored in system keychain

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vue 3 + TypeScript + Pinia |
| Terminal | xterm.js with fit/web-links addons |
| Backend | Rust with Tauri 2.0 |
| PTY | portable-pty for cross-platform terminal |
| LLM | Ollama / OpenAI-compatible APIs |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      aiTerm Architecture                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐  │
│  │  Vue 3 UI   │    │  xterm.js   │    │   AI Chat Panel │  │
│  │  (Pinia)    │    │  Terminal   │    │                 │  │
│  └──────┬──────┘    └──────┬──────┘    └────────┬────────┘  │
│         │                  │                     │           │
│         └──────────────────┼─────────────────────┘           │
│                            │                                 │
│                   Tauri IPC (invoke/events)                  │
│                            │                                 │
├────────────────────────────┼─────────────────────────────────┤
│  ┌─────────────┐    ┌──────┴──────┐    ┌─────────────────┐  │
│  │ PTY Manager │    │  LLM Client │    │ Context Manager │  │
│  │ (portable)  │    │ (Ollama/    │    │ (Ring Buffer +  │  │
│  │             │    │  OpenAI)    │    │  Summarization) │  │
│  └─────────────┘    └─────────────┘    └─────────────────┘  │
│                     Rust Backend                             │
└─────────────────────────────────────────────────────────────┘
```

## Context Management

aiTerm uses a sophisticated context management system to provide the AI with relevant terminal history:

- **Sliding Window**: Keeps last 500 lines of terminal I/O
- **Token Budget**: Respects max token limits when building context
- **LLM Summarization**: Old entries are compressed into summaries
- **Key Info Preservation**: Errors and important commands are retained

## Installation

### Download

Download the latest release from [Releases](https://github.com/yourusername/aiTerm/releases) page.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/aiTerm.git
cd aiTerm

# Install dependencies
pnpm install

# Development
pnpm tauri dev

# Build
pnpm tauri build
```

## Configuration

### Ollama (Local)

1. Install Ollama: https://ollama.ai
2. Pull a model: `ollama pull llama3.2`
3. Open aiTerm Settings (Cmd+,)
4. Select "Ollama" as provider
5. Set model name (e.g., `llama3.2`)

### OpenAI / Compatible APIs

1. Open aiTerm Settings (Cmd+,)
2. Select "OpenAI" as provider
3. Enter your API key (stored securely in system keychain)
4. Optionally change API base URL for compatible services

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+,` | Open Settings |
| `Escape` | Close modal dialogs |
| `Enter` | Send AI message |

### AI Commands

The AI assistant can:
- Explain commands and their options
- Help debug error messages
- Suggest commands for tasks
- Analyze terminal output

When the AI suggests a command, you can:
1. Click the command button to execute
2. Edit the command before execution
3. Cancel if not needed

## Development

### Project Structure

```
aiTerm/
├── src/                    # Vue frontend
│   ├── components/         # Vue components
│   │   ├── Terminal.vue    # xterm.js wrapper
│   │   ├── AIChat.vue      # AI chat panel
│   │   ├── Settings.vue    # Settings modal
│   │   └── CmdConfirm.vue  # Command confirmation
│   ├── stores/             # Pinia stores
│   │   ├── ai.ts           # AI state
│   │   ├── settings.ts     # App settings
│   │   └── terminal.ts     # Terminal state
│   └── App.vue             # Main app
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri commands
│   │   ├── pty/            # PTY management
│   │   ├── llm/            # LLM client
│   │   ├── context/        # Context manager
│   │   └── config/         # Configuration
│   └── Cargo.toml
└── package.json
```

### Key Backend Modules

- **pty/mod.rs** - PTY session management with portable-pty
- **llm/mod.rs** - Unified LLM client for Ollama/OpenAI
- **context/mod.rs** - Ring buffer + summarization
- **config/mod.rs** - Persistent configuration

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop apps
- [xterm.js](https://xtermjs.org/) - Terminal emulator
- [Vue 3](https://vuejs.org/) - Frontend framework
- [Ollama](https://ollama.ai/) - Local LLM runtime
