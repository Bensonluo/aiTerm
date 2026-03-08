# aiTerm 研究发现

## 技术调研

### 1. 终端模拟器框架对比

| 框架 | 语言 | 跨平台 | 成熟度 | 适用性 |
|------|------|--------|--------|--------|
| **xterm.js** | TypeScript | ✅ | ⭐⭐⭐⭐⭐ | 最佳选择，VS Code 终端使用 |
| alacritty_terminal | Rust | ✅ | ⭐⭐⭐⭐ | 仅 PTY 层，需自己渲染 |
| console_widget | Flutter | ✅ | ⭐⭐ | 不够成熟 |

**结论**: xterm.js 是最成熟的选择，配合 Tauri IPC 通信。

### 2. PTY 管理库对比

| 库 | 语言 | 跨平台 | 维护状态 |
|-----|------|--------|----------|
| **portable-pty** | Rust | ✅ | 活跃 |
| alacritty_terminal | Rust | ✅ | 活跃 |
| node-pty | Node.js | ✅ | 活跃 |

**结论**: portable-pty API 简洁，适合 Tauri 后端。

### 3. Ollama API

**端点**: `http://localhost:11434/api/chat`

**请求格式**:
```json
{
  "model": "llama3.2",
  "messages": [
    {"role": "user", "content": "..."}
  ],
  "stream": true
}
```

**响应格式 (SSE)**:
```json
{"model":"llama3.2","created_at":"2024-...","message":{"role":"assistant","content":"片段"},"done":false}
```

**流式处理**: 使用 `reqwest` + `futures` 处理 SSE。

### 4. 类似项目参考

| 项目 | 特点 | 可借鉴 |
|------|------|--------|
| **Warp** | AI-First 终端 | UX 设计、命令块识别 |
| **Alacritty** | 高性能终端 | PTY 处理、GPU 渲染 |
| **kitty** | 功能丰富 | 终端协议实现 |
| **Fig** (已关闭) | 补全功能 | 补全 UI 设计 |

---

## 设计决策

### 为什么选 Tauri 而非 Electron？

| 维度 | Tauri | Electron |
|------|-------|----------|
| 内存占用 | ~100MB | ~300MB |
| 安装包大小 | ~10MB | ~50MB |
| 启动速度 | 快 | 较慢 |
| Rust 生态 | 原生 | 需要 FFI |

### 为什么仅支持 Ollama？

1. **简化 MVP** - 减少适配工作量
2. **统一 API** - Ollama 接口稳定
3. **后续扩展** - 可通过抽象层支持其他后端

### 为什么用确认模式而非自动执行？

1. **安全性** - 防止误执行危险命令
2. **可控性** - 用户有最终决定权
3. **信任建立** - 初期建立用户信任

---

## Prompt 工程

### System Prompt (运维场景)

```
你是运维助手，根据终端历史帮助用户解决问题。

规则：
1. 只给出可直接执行的命令，用 ``` 包裹
2. 简短解释原因（1-2 句）
3. 不确定时询问更多信息
4. 危险操作（rm、sudo 等）需特别提醒
5. 优先使用用户已使用的工具（如 kubectl）

输出格式：
命令：
```bash
<命令>
```

原因：<简短解释>
```

### 上下文截断策略

- 保留最近 500 行
- 优先保留错误信息和用户输入
- 超过 token 限制时，保留最近 50 行 + 错误上下文

---

## 调研结论 (2026-03-03)

### Ratatui vs xterm.js

**Ratatui** 是 TUI (Terminal User Interface) 框架，用于在终端内运行的应用（类似 lazygit/h top）。
**xterm.js** 是终端模拟器渲染库，用于构建运行 shell 的桌面应用。

本项目需要的是后者，因此 xterm.js 是正确选择。

### EFx (egui 模板引擎)

EFx 是 egui 的 XML 风格模板引擎，可简化 egui UI 代码。但由于我们选择 Tauri + Vue 前端，egui 不适用。

---

## 待研究

- [ ] xterm.js 性能优化（大数据量输出）
- [ ] Windows PTY 兼容性测试
- [ ] 多标签页实现方案
- [ ] 自动更新机制细节
