# aiTerm 实施计划

## 项目目标

构建 AI-First 本地终端，类似 Warp 但纯本地运行，仅连接本地 Ollama 模型。

**核心指标**：
- MVP 工期：23-36 天
- 完整产品：40-55 天
- 跨平台支持：macOS / Windows / Linux

---

## 里程碑概览

| 阶段 | 名称 | 工期 | 交付物 |
|------|------|------|--------|
| M1 | 项目骨架 | 2-3 天 | Tauri 项目初始化、基础目录结构 |
| M2 | 终端核心 | 5-8 天 | 可用的终端模拟器 |
| M3 | AI 集成 | 7-10 天 | Ollama 连通、上下文管理 |
| M4 | AI UI | 7-11 天 | 对话面板、命令确认 |
| M5 | 基础设施 | 4-7 天 | 设置、打包、错误处理 |
| M6 | MVP 打磨 | 5-7 天 | Bug 修复、体验优化 |

**总工期**：30-46 天（约 1.5-2 个月）

---

## Phase 1: 项目骨架 (M1)

**状态**: `pending`
**工期**: 2-3 天
**优先级**: P0 (阻塞)

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 1.1 | 初始化 Tauri 2.0 项目 | P0 | - | `pnpm tauri dev` 可启动 |
| 1.2 | 配置 Vue 3 + TypeScript | P0 | 1.1 | Vite 构建成功 |
| 1.3 | 配置 Rust 后端基础结构 | P0 | 1.1 | Cargo 编译通过 |
| 1.4 | 配置 ESLint + Prettier | P1 | 1.2 | Lint 检查通过 |
| 1.5 | 配置 rust-analyzer + clippy | P1 | 1.3 | IDE 提示正常 |
| 1.6 | 创建目录结构 | P0 | 1.1 | 见下方结构 |

### 目录结构

```
aiTerm/
├── src/                    # Vue 前端
│   ├── components/
│   │   ├── Terminal.vue
│   │   ├── AIChat.vue
│   │   └── Settings.vue
│   ├── stores/
│   │   └── terminal.ts
│   ├── App.vue
│   └── main.ts
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── pty/
│   │   ├── ai/
│   │   ├── config/
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/
└── package.json
```

### 交付物
- [ ] 可运行的 Tauri 空白应用
- [ ] 完整的项目结构
- [ ] 开发环境配置文档

---

## Phase 2: 终端核心 (M2)

**状态**: `pending`
**工期**: 5-8 天
**优先级**: P0 (核心功能)
**依赖**: Phase 1

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 2.1 | 集成 xterm.js | P0 | 1.2 | 终端组件渲染 |
| 2.2 | 实现 pty_create 命令 | P0 | 1.3 | shell 进程启动 |
| 2.3 | 实现 pty_write 命令 | P0 | 2.2 | 用户输入传递 |
| 2.4 | 实现 pty_resize 命令 | P0 | 2.2 | 窗口大小调整 |
| 2.5 | 实现 pty_output 事件 | P0 | 2.2 | 输出流式显示 |
| 2.6 | 实现上下文捕获 | P1 | 2.5 | 历史记录存储 |
| 2.7 | 终端主题配置 | P2 | 2.1 | 深色/浅色主题 |
| 2.8 | 字体/字号配置 | P2 | 2.1 | 可调节 |

### Rust 模块设计

```rust
// src-tauri/src/pty/mod.rs
pub struct PtyManager {
    session: Option<Box<dyn PtySession>>,
    context_buffer: VecDeque<String>,
}

impl PtyManager {
    pub fn create(shell: &str) -> Result<Self>;
    pub fn write(&mut self, data: &str) -> Result<()>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn read_output(&mut self) -> Option<String>;
}
```

### 交付物
- [ ] 可交互的终端模拟器
- [ ] shell 命令执行正常
- [ ] 上下文历史捕获

---

## Phase 3: AI 集成 (M3)

**状态**: `pending`
**工期**: 7-10 天
**优先级**: P0 (核心功能)
**依赖**: Phase 2

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 3.1 | 实现 OllamaClient | P0 | 2.6 | HTTP 连接成功 |
| 3.2 | 实现流式响应解析 | P0 | 3.1 | SSE 正常解析 |
| 3.3 | 实现 ContextManager | P0 | 2.6 | 环形缓冲正常 |
| 3.4 | 实现 Prompt 模板 | P0 | 3.3 | 模板渲染正确 |
| 3.5 | 实现 ai_chat 命令 | P0 | 3.1, 3.3 | 前端可调用 |
| 3.6 | 实现 ai_stream 事件 | P0 | 3.5 | 流式输出正常 |
| 3.7 | 错误处理（连接失败） | P1 | 3.1 | 友好提示 |
| 3.8 | 模型列表获取 | P1 | 3.1 | 下拉选择 |

### Rust 模块设计

```rust
// src-tauri/src/ai/mod.rs
pub struct OllamaClient {
    host: String,
    model: String,
}

pub struct ContextManager {
    buffer: VecDeque<TerminalEntry>,
    max_lines: usize,
}

pub struct PromptTemplate {
    system: String,
}

impl OllamaClient {
    pub async fn chat(&self, messages: Vec<Message>) -> impl Stream<Item = String>;
    pub async fn list_models(&self) -> Result<Vec<String>>;
}

impl ContextManager {
    pub fn push(&mut self, entry: TerminalEntry);
    pub fn build_context(&self, max_tokens: usize) -> String;
}
```

### 交付物
- [ ] Ollama API 连通
- [ ] 上下文管理正常
- [ ] 流式响应正常

---

## Phase 4: AI UI (M4)

**状态**: `pending`
**工期**: 7-11 天
**优先级**: P0 (核心功能)
**依赖**: Phase 3

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 4.1 | AI 对话面板 UI | P0 | 3.5 | 聊天界面显示 |
| 4.2 | 流式输出渲染 | P0 | 3.6 | 打字机效果 |
| 4.3 | 命令识别（正则） | P0 | 4.2 | 识别命令块 |
| 4.4 | 命令确认弹窗 | P0 | 4.3 | y/n/e 交互 |
| 4.5 | 命令执行（确认后） | P0 | 4.4 | 写入 PTY |
| 4.6 | 快捷键绑定 (Ctrl+Space) | P1 | 4.1 | 触发 AI |
| 4.7 | 行内补全提示 | P2 | 4.2 | 灰色提示 |
| 4.8 | 错误自动建议 | P2 | 4.2 | 检测错误 |

### 前端组件设计

```vue
<!-- src/components/AIChat.vue -->
<template>
  <div class="ai-panel">
    <div class="messages">
      <Message v-for="msg in messages" :key="msg.id" :message="msg" />
    </div>
    <div class="input-area">
      <input v-model="userInput" @keyup.enter="send" />
    </div>
  </div>
</template>
```

```vue
<!-- src/components/CmdConfirm.vue -->
<template>
  <div class="confirm-modal">
    <div class="command">{{ command }}</div>
    <div class="reason">{{ reason }}</div>
    <div class="actions">
      <button @click="cancel">取消 (n)</button>
      <button @click="edit">编辑 (e)</button>
      <button @click="execute">执行 (y)</button>
    </div>
  </div>
</template>
```

### 交付物
- [ ] 完整的 AI 对话界面
- [ ] 命令确认流程
- [ ] 快捷键交互

---

## Phase 5: 基础设施 (M5)

**状态**: `pending`
**工期**: 4-7 天
**优先级**: P1 (必需)
**依赖**: Phase 4

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 5.1 | 配置文件读写 | P0 | - | JSON 持久化 |
| 5.2 | 设置界面 UI | P1 | 5.1 | 可视化配置 |
| 5.3 | Ollama 连接检测 | P0 | 5.1 | 启动时检查 |
| 5.4 | 错误提示 UI | P1 | 5.3 | 友好提示 |
| 5.5 | macOS 打包 | P0 | - | .dmg 生成 |
| 5.6 | Windows 打包 | P1 | - | .msi 生成 |
| 5.7 | Linux 打包 | P1 | - | .deb 生成 |
| 5.8 | 自动更新机制 | P2 | 5.5 | Tauri updater |

### 配置结构

```json
// ~/.aiterm/config.json
{
  "ollama": {
    "host": "http://localhost:11434",
    "model": "llama3.2"
  },
  "terminal": {
    "shell": "auto",
    "fontSize": 14,
    "theme": "dark"
  },
  "context": {
    "maxLines": 500,
    "maxTokens": 4096
  }
}
```

### 交付物
- [ ] 配置持久化
- [ ] 设置界面
- [ ] 三平台安装包

---

## Phase 6: MVP 打磨 (M6)

**状态**: `pending`
**工期**: 5-7 天
**优先级**: P1 (质量)
**依赖**: Phase 5

### 任务清单

| # | 任务 | 优先级 | 依赖 | 验收标准 |
|---|------|--------|------|----------|
| 6.1 | Bug 修复 | P0 | - | 无阻塞性 Bug |
| 6.2 | 性能优化 | P1 | - | 响应 < 100ms |
| 6.3 | UI 动画 | P2 | - | 流畅过渡 |
| 6.4 | 快捷键完善 | P1 | - | 常用操作 |
| 6.5 | 文档编写 | P1 | - | README + Wiki |
| 6.6 | 单元测试 | P2 | - | 核心覆盖 |
| 6.7 | 集成测试 | P2 | - | E2E 流程 |

### 交付物
- [ ] 稳定的 MVP 版本
- [ ] 用户文档
- [ ] 发布包

---

## 依赖关系图

```
M1 (项目骨架)
 │
 ├──▶ M2 (终端核心)
 │     │
 │     └──▶ M3 (AI 集成)
 │           │
 │           └──▶ M4 (AI UI)
 │                 │
 │                 └──▶ M5 (基础设施)
 │                       │
 │                       └──▶ M6 (打磨)
 │
 └──▶ M5 (基础设施) [部分并行]
```

---

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| portable-pty 跨平台兼容 | 高 | 早期测试三平台 |
| Ollama API 变更 | 中 | 抽象 API 层 |
| xterm.js 性能 | 中 | 限制缓冲区大小 |
| LLM 响应延迟 | 中 | 优化 prompt 长度 |

---

## 决策记录

| 日期 | 决策 | 原因 |
|------|------|------|
| 2026-03-03 | 选择 Tauri + Vue | 跨平台、前端灵活、xterm.js 成熟 |
| 2026-03-03 | 仅支持 Ollama | 简化 MVP，后续可扩展 |
| 2026-03-03 | 确认模式执行命令 | 安全性优先 |

---

## 错误记录

| 错误 | 尝试 | 解决方案 |
|------|------|----------|
| (待记录) | - | - |
