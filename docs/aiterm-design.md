# aiTerm 设计文档

## 概述

AI-First 本地终端，类似 Warp 但纯本地运行，仅连接本地 Ollama 模型。

### 核心特性
- 原生 GUI 桌面应用（跨平台：macOS/Windows/Linux）
- 基于终端上下文的运维 AI 助手
- 用户确认后执行命令（y/n 模式）
- 仅支持 Ollama 本地模型

### 目标用户
- 运维工程师
- 需要在服务器环境排查问题的开发者
- 注重隐私、不希望数据上云的用户

---

## 技术栈

| 层级 | 技术选型 |
|------|----------|
| 前端框架 | Vue 3 + TypeScript |
| 终端渲染 | xterm.js |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| PTY 管理 | portable-pty |
| LLM 调用 | Ollama HTTP API |
| 配置存储 | JSON 文件 |

---

## 架构设计

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      用户界面层 (Vue)                        │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
│  │  TerminalView │  │   AIChatPanel │  │  CmdConfirm   │   │
│  │  (xterm.js)   │  │   (对话UI,浮窗 )     │  │  (确认弹窗)   │   │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘   │
└──────────┼──────────────────┼──────────────────┼───────────┘
           │                  │                  │
           └──────────────────┼──────────────────┘
                              │ Tauri Commands (IPC)
┌─────────────────────────────┼───────────────────────────────┐
│                      Rust 后端                               │
│  ┌───────────────┐  ┌───────┴───────┐  ┌───────────────┐   │
│  │  PtyManager   │  │ ContextManager │  │ OllamaClient  │   │
│  │  (进程管理)    │  │ (上下文缓冲)    │  │ (LLM调用)     │   │
│  └───────┬───────┘  └───────────────┘  └───────┬───────┘   │
│          │                                      │           │
│          └──────────── Event Loop ──────────────┘           │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │    Ollama Server   │
                    │   (localhost:11434)│
                    └────────────────────┘
```

### 核心设计原则
- 前端负责 UI 渲染和用户交互
- Rust 后端负责 PTY 管理、LLM 调用、上下文状态
- 事件驱动架构，通过 Tauri IPC 通信

---

## 模块详细设计

### 1. 终端核心模块

**数据流**：
```
用户输入 → xterm.js → pty_write → PtyManager → shell
shell 输出 → PtyManager → event:pty_output → xterm.js 渲染
```

**Tauri IPC Commands**：
| Command | 参数 | 说明 |
|---------|------|------|
| `pty_create` | shell: string | 创建新的 PTY 会话 |
| `pty_write` | data: string | 向 PTY 写入数据 |
| `pty_resize` | cols, rows | 调整终端大小 |

**Tauri Events**：
| Event | 数据 | 说明 |
|-------|------|------|
| `pty_output` | data: string | PTY 输出数据 |
| `pty_exit` | code: number | shell 退出 |

**上下文捕获**：
- 每条输出同时发送给 `ContextManager`
- 按时间/行数限制缓冲（默认 500 行或 10 分钟）
- 区分命令输入和输出内容

### 2. AI 集成模块

**ContextManager**：
```
Ring Buffer (500 行)
┌─────┬─────┬─────┬─────┬─────┐
│cmd1 │out1 │cmd2 │out2 │ ... │
└─────┴─────┴─────┴─────┴─────┘

build_prompt(user_input) → String
- 取最近 N 行历史
- 拼接 system prompt + 历史 + 用户问题
```

**OllamaClient**：
```
POST http://localhost:11434/api/chat

Request:
{
  "model": "llama3.2",
  "messages": [...],
  "stream": true
}

Response (SSE stream):
{ "message": { "content": "片段" } }

→ 流式 emit 到前端渲染
```

**Prompt 模板**：
```
[System]
你是运维助手，根据终端历史帮助用户解决问题。
- 只给出可直接执行的命令
- 简短解释原因（1-2 句）
- 不确定时询问更多信息

[终端历史]
$ kubectl get pods
NAME                    READY   STATUS    RESTARTS
nginx-deployment-abc    0/1     CrashLoopBackOff   5

$ kubectl logs nginx-deployment-abc
Error: connection refused to database

[用户问题]
这个 pod 怎么一直重启？
```

### 3. AI UI & 交互模块

**主窗口布局**：
```
┌───────────────────────────────────┬─────────────────┐
│                                   │                 │
│        终端区域                    │    AI 对话面板  │
│        (xterm.js)                 │    (可折叠侧窗或浮窗)     │
│                                   │                 │
│  $ kubectl get pods               │  ┌───────────┐  │
│  NAME              STATUS         │  │ 用户: ... │  │
│  nginx-xxx         CrashLoop      │  │ AI: ...   │  │
│                                   │  │           │  │
│  $ ▌  [灰色补全提示]              │  └───────────┘  │
│                                   │  [输入框]       │
└───────────────────────────────────┴─────────────────┘
```

**交互流程**：
```
触发方式              AI 响应              用户确认
    │                    │                    │
    ▼                    ▼                    ▼
1. 快捷键           1. 流式输出         1. 按 y 执行
   Ctrl+Space           到对话面板        2. 按 n 取消
                          或
2. 侧栏输入          2. 如果是命令      3. 按 e 编辑
   直接提问             弹出确认框           后执行

3. 错误时自动
   触发建议
```

**命令确认弹窗**：
```
┌─────────────────────────────────────┐
│  AI 建议执行以下命令:                │
│  ┌───────────────────────────────┐  │
│  │ kubectl describe pod nginx-xxx│  │
│  └───────────────────────────────┘  │
│                                     │
│  原因: 查看 Pod 详细状态和事件       │
│                                     │
│     [取消]  [编辑]  [执行]          │
│                (y)    (Enter)       │
└─────────────────────────────────────┘
```

### 4. 基础设施模块

**配置文件 (JSON)**：
```json
{
  "ollama": {
    "host": "http://localhost:11434",
    "model": "llama3.2"
  },
  "terminal": {
    "shell": "/bin/zsh",
    "fontSize": 14,
    "theme": "dark"
  },
  "context": {
    "maxLines": 500,
    "maxTokens": 4096
  }
}
```

**错误处理**：
- Ollama 连接失败 → 显示重试/设置引导
- 模型不存在 → 提示下载命令
- Shell 退出异常 → 显示退出码

**跨平台打包**：
| 平台 | 格式 |
|------|------|
| macOS | .dmg, .app |
| Windows | .msi, .exe |
| Linux | .deb, .AppImage |

---

## 工作量估算

### MVP (基础可用) - 23-36 天

| 模块 | 工作量 | 说明 |
|------|--------|------|
| 终端核心 | 5-8 天 | PTY、xterm.js、基础渲染 |
| AI 集成 | 7-10 天 | Ollama 客户端、上下文管理、Prompt |
| AI UI | 7-11 天 | 对话面板、补全、确认弹窗 |
| 基础设施 | 4-7 天 | 设置、打包、错误处理 |

### 完整产品 (v1.0) - 40-55 天

MVP + 额外打磨：
- 体验优化（动画、快捷键）
- 更多终端功能（分屏、标签）
- 高级 AI 功能（自动建议、错误诊断）

---

## 依赖清单

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
portable-pty = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
futures = "0.3"
```

### 前端 (package.json)
```json
{
  "dependencies": {
    "vue": "^3.4",
    "xterm": "^5.3",
    "xterm-addon-fit": "^0.8",
    "xterm-addon-web-links": "^0.9"
  }
}
```

---

## 后续迭代方向 (v2+)

1. **多会话关联** - 多个终端标签页共享上下文
2. **命令历史同步** - 跨设备同步（可选云存储）
3. **更多模型支持** - llama.cpp server 等
4. **插件系统** - 自定义 AI 命令/工作流
5. **团队协作** - 共享终端会话（需自建服务）
