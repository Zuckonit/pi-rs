# Pi Coding Agent Rust 实现计划

> **For Claude:** REQUIRED SUB-SKILL: 使用 superpowers:executing-plans 来实现此计划

**目标:** 用 Rust 完整实现 pi-coding-agent 的所有功能

**架构:** 
- 核心库 (`pi-core`): 会话管理、工具系统、LLM Provider
- TUI 应用 (`pi-tui`): 交互式终端UI
- CLI 应用 (`pi`): 命令行入口

**技术栈:**
- async-std/tokio: 异步运行时
- ratatui: 终端UI
- serde: 序列化
- reqwest: HTTP客户端

---

## 阶段1: 核心库 (Core Library)

### Task 1.1: 项目初始化和基础类型

**文件:**
- 创建: `pi-core/Cargo.toml`
- 创建: `pi-core/src/lib.rs`
- 创建: `pi-core/src/types/mod.rs`
- 创建: `pi-core/src/types/message.rs`
- 创建: `pi-core/src/types/model.rs`

**步骤1: 创建项目结构**

```bash
cargo new --lib pi-core
cd pi-core
```

**步骤2: 定义消息类型**

```rust
// src/types/message.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}
```

**步骤3: 定义模型类型**

```rust
// src/types/model.rs
#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: u64,
    pub max_tokens: u64,
    pub supports_thinking: bool,
}
```

**步骤4: 提交**

```bash
git add -A && git commit -m "feat: add core types"
```

---

### Task 1.2: 会话管理系统

**文件:**
- 创建: `pi-core/src/session/mod.rs`
- 创建: `pi-core/src/session/entry.rs`
- 创建: `pi-core/src/session/manager.rs`
- 测试: `pi-core/tests/session_test.rs`

**步骤1: 定义会话入口类型**

```rust
// src/session/entry.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    pub r#type: String,
    pub version: Option<u32>,
    pub id: String,
    pub timestamp: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionEntry {
    Message(MessageEntry),
    ThinkingLevelChange(ThinkingLevelChangeEntry),
    ModelChange(ModelChangeEntry),
    Compaction(CompactionEntry),
    BranchSummary(BranchSummaryEntry),
    Custom(CustomEntry),
    CustomMessage(CustomMessageEntry),
    Label(LabelEntry),
    SessionInfo(SessionInfoEntry),
}
```

**步骤2: 实现会话管理器**

```rust
// src/session/manager.rs
pub struct SessionManager {
    session_id: String,
    session_file: Option<PathBuf>,
    file_entries: Vec<FileEntry>,
    by_id: HashMap<String, SessionEntry>,
    leaf_id: Option<String>,
}

impl SessionManager {
    pub fn create(cwd: &str, session_dir: Option<&Path>) -> Self
    pub fn open(path: &Path) -> Self
    pub fn append_message(&mut self, message: Message) -> String
    pub fn get_branch(&self, from_id: Option<&str>) -> Vec<&SessionEntry>
    pub fn build_session_context(&self) -> SessionContext
    pub fn branch(&mut self, branch_from_id: &str) -> Result<(), String>
    pub fn compact(&mut self, summary: &str, first_kept_id: &str) -> String
}
```

**步骤3: 编写测试**

```rust
// tests/session_test.rs
#[test]
fn test_session_create() {
    let session = SessionManager::in_memory("/tmp");
    assert!(session.get_session_id().len() > 0);
}

#[test]
fn test_session_append_message() {
    let mut session = SessionManager::in_memory("/tmp");
    let id = session.append_message(Message::user("hello"));
    assert!(!id.is_empty());
}
```

**步骤4: 运行测试**

```bash
cd pi-core && cargo test
```

**步骤5: 提交**

```bash
git add -A && git commit -m "feat: add session management"
```

---

### Task 1.3: 工具系统

**文件:**
- 创建: `pi-core/src/tools/mod.rs`
- 创建: `pi-core/src/tools/tool.rs`
- 创建: `pi-core/src/tools/read.rs`
- 创建: `pi-core/src/tools/write.rs`
- 创建: `pi-core/src/tools/edit.rs`
- 创建: `pi-core/src/tools/bash.rs`
- 创建: `pi-core/src/tools/grep.rs`
- 创建: `pi-core/src/tools/find.rs`
- 创建: `pi-core/src/tools/ls.rs`

**步骤1: 定义工具trait**

```rust
// src/tools/tool.rs
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn execute(&self, args: serde_json::Value, cwd: &str) -> Result<ToolResult, ToolError>;
}
```

**步骤2: 实现Read工具**

```rust
// src/tools/read.rs
pub struct ReadTool;

impl Tool for ReadTool {
    async fn execute(&self, args: serde_json::Value, cwd: &str) -> Result<ToolResult, ToolError> {
        let path = args["path"].as_str().ok_or("missing path")?;
        let content = tokio::fs::read_to_string(Path::new(cwd).join(path)).await?;
        Ok(ToolResult::success(content))
    }
}
```

**步骤3: 实现其他工具**

类似实现 WriteTool, EditTool, BashTool, GrepTool, FindTool, LsTool

**步骤4: 提交**

```bash
git add -A && git commit -m "feat: add tool system"
```

---

### Task 1.4: LLM Provider 系统

**文件:**
- 创建: `pi-core/src/providers/mod.rs`
- 创建: `pi-core/src/providers/trait.rs`
- 创建: `pi-core/src/providers/anthropic.rs`
- 创建: `pi-core/src/providers/openai.rs`
- 创建: `pi-core/src/providers/google.rs`
- 创建: `pi-core/src/providers/registry.rs`

**步骤1: 定义Provider trait**

```rust
// src/providers/trait.rs
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<Model>;
    
    async fn chat(&self, model: &str, messages: Vec<Message>, 
                  tools: Option<Vec<Tool>>) -> Result<ChatResponse, ProviderError>;
                  
    async fn chat_stream(&self, model: &str, messages: Vec<Message>,
                         tools: Option<Vec<Tool>>) -> Result<BoxStream<ChatChunk>, ProviderError>;
}
```

**步骤2: 实现Anthropic Provider**

```rust
// src/providers/anthropic.rs
pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn chat(&self, model: &str, messages: Vec<Message>, 
                  tools: Option<Vec<Tool>>) -> Result<ChatResponse, ProviderError> {
        // 调用 Anthropic API
    }
}
```

**步骤3: 实现OpenAI和Google Provider**

类似实现

**步骤4: 实现ModelRegistry**

```rust
// src/providers/registry.rs
pub struct ModelRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ModelRegistry {
    pub fn register(&mut self, name: &str, provider: Arc<dyn Provider>)
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>>
    pub fn get_model(&self, model_id: &str) -> Option<Model>
}
```

**步骤5: 提交**

```bash
git add -A && git commit -m "feat: add LLM providers"
```

---

### Task 1.5: 设置管理系统

**文件:**
- 创建: `pi-core/src/settings/mod.rs`
- 创建: `pi-core/src/settings/manager.rs`

**步骤1: 定义设置结构**

```rust
// src/settings/manager.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub thinking_level: String,
    pub theme: String,
    pub steering_mode: String,
    pub follow_up_mode: String,
    pub auto_compact: bool,
    pub compaction_threshold: f64,
    // ... 更多设置
}

pub struct SettingsManager {
    global_settings: Settings,
    project_settings: Option<Settings>,
    path: PathBuf,
}
```

**步骤2: 实现设置加载**

```rust
impl SettingsManager {
    pub fn load(cwd: &str) -> Self
    pub fn get(&self, key: &str) -> serde_json::Value
    pub fn set(&mut self, key: &str, value: serde_json::Value) -> Result<(), Error>
    pub fn save(&self) -> Result<(), Error>
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add settings manager"
```

---

### Task 1.6: 认证系统

**文件:**
- 创建: `pi-core/src/auth/mod.rs`
- 创建: `pi-core/src/auth/storage.rs`

**步骤1: 定义认证类型**

```rust
// src/auth/storage.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Credential {
    ApiKey { key: String },
    OAuth { access_token: String, refresh_token: String, expires_at: i64 },
}

pub struct AuthStorage {
    credentials: HashMap<String, Credential>,
    path: PathBuf,
}

impl AuthStorage {
    pub fn load(path: &Path) -> Self
    pub fn get(&self, provider: &str) -> Option<&Credential>
    pub fn set(&mut self, provider: &str, credential: Credential)
    pub fn save(&self) -> Result<(), Error>
    pub fn get_api_key(&self, provider: &str) -> Option<String>
}
```

**步骤2: 提交**

```bash
git add -A && git commit -m "feat: add auth system"
```

---

## 阶段2: Agent Core

### Task 2.1: Agent 实现

**文件:**
- 创建: `pi-core/src/agent/mod.rs`
- 创建: `pi-core/src/agent/session.rs`
- 创建: `pi-core/src/agent/loop.rs`

**步骤1: 定义Agent Session**

```rust
// src/agent/session.rs
pub struct AgentSession {
    session: SessionManager,
    provider: Arc<dyn Provider>,
    model: Model,
    tools: Vec<Arc<dyn Tool>>,
    settings: SettingsManager,
    event_bus: EventBus,
}

impl AgentSession {
    pub fn new(config: AgentConfig) -> Self
    
    pub async fn prompt(&mut self, text: &str) -> Result<(), Error>
    pub async fn steer(&mut self, text: &str) -> Result<(), Error>
    pub async fn follow_up(&mut self, text: &str) -> Result<(), Error>
    
    pub fn set_model(&mut self, model: Model) -> Result<(), Error>
    pub fn set_thinking_level(&mut self, level: ThinkingLevel)
    
    pub fn on_event<F>(&mut self, event: &str, handler: F)
}
```

**步骤2: 实现Agent循环**

```rust
// src/agent/loop.rs
impl AgentSession {
    pub async fn run_loop(&mut self) -> Result<(), Error> {
        loop {
            // 1. 获取上下文
            let context = self.session.build_session_context();
            
            // 2. 调用LLM
            let response = self.provider.chat(&self.model.id, context.messages, Some(self.tools.clone())).await?;
            
            // 3. 处理工具调用
            for tool_call in response.tool_calls {
                let result = self.execute_tool(&tool_call).await?;
                self.session.append_tool_result(tool_call.id, result);
            }
            
            // 4. 检查是否需要压缩
            if self.should_compact() {
                self.compact().await?;
            }
        }
    }
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add agent core"
```

---

## 阶段3: TUI 应用

### Task 3.1: TUI 基础框架

**文件:**
- 创建: `pi-tui/Cargo.toml`
- 创建: `pi-tui/src/main.rs`
- 创建: `pi-tui/src/app.rs`
- 创建: `pi-tui/src/components/mod.rs`

**步骤1: 创建项目**

```bash
cargo new pi-tui
# 添加依赖: ratatui, crosstrm, tokio
```

**步骤2: 实现应用结构**

```rust
// src/app.rs
pub struct App {
    terminal: Terminal,
    session: AgentSession,
    state: AppState,
}

pub enum AppState {
    Idle,
    Thinking,
    ToolExecuting,
    WaitingInput,
}

impl App {
    pub fn new(session: AgentSession) -> Self
    pub async fn run(&mut self) -> Result<(), Error>
    fn handle_input(&mut self, key: Key) -> Result<(), Error>
    fn render(&mut self)
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add TUI framework"
```

---

### Task 3.2: 消息渲染组件

**文件:**
- 创建: `pi-tui/src/components/messages.rs`
- 创建: `pi-tui/src/components/editor.rs`
- 创建: `pi-tui/src/components/footer.rs`

**步骤1: 实现消息列表**

```rust
// src/components/messages.rs
pub struct MessageList {
    messages: Vec<Message>,
    scroll_offset: usize,
}

impl MessageList {
    pub fn new() -> Self
    pub fn add_message(&mut self, message: Message)
    pub fn render(&self, area: Rect, buf: &mut Buffer)
    pub fn scroll_up(&mut self)
    pub fn scroll_down(&mut self)
}
```

**步骤2: 实现编辑器**

```rust
// src/components/editor.rs
pub struct Editor {
    content: String,
    cursor_position: usize,
    history: Vec<String>,
}

impl Editor {
    pub fn new() -> Self
    pub fn insert_char(&mut self, c: char)
    pub fn delete_char(&mut self)
    pub fn move_cursor(&mut self, direction: Direction)
    pub fn submit(&mut self) -> String
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add UI components"
```

---

### Task 3.3: 交互逻辑

**文件:**
- 创建: `pi-tui/src/input/mod.rs`
- 创建: `pi-tui/src/input/handler.rs`
- 创建: `pi-tui/src/input/completion.rs`

**步骤1: 实现输入处理**

```rust
// src/input/handler.rs
pub struct InputHandler {
    editor: Editor,
    completion: CompletionEngine,
}

impl InputHandler {
    pub fn handle_key(&mut self, key: Key) -> InputAction
    pub fn get_completions(&self) -> Vec<Completion>
}
```

**步骤2: 实现自动补全**

```rust
// src/input/completion.rs
pub struct CompletionEngine {
    commands: Vec<Command>,
    skills: Vec<Skill>,
    prompts: Vec<PromptTemplate>,
}

impl CompletionEngine {
    pub fn get_completions(&self, input: &str) -> Vec<Completion>
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add input handling"
```

---

### Task 3.4: 主题系统

**文件:**
- 创建: `pi-tui/src/theme/mod.rs`
- 创建: `pi-tui/src/theme/parser.rs`

**步骤1: 定义主题**

```rust
// src/theme/mod.rs
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
    // ... 更多颜色
}
```

**步骤2: 提交**

```bash
git add -A && git commit -m "feat: add theme system"
```

---

### Task 3.5: 扩展系统集成

**文件:**
- 创建: `pi-tui/src/extensions/mod.rs`
- 创建: `pi-tui/src/extensions/loader.rs`
- 创建: `pi-tui/src/extensions/api.rs`

**步骤1: 实现扩展加载器**

```rust
// src/extensions/loader.rs
pub struct ExtensionLoader {
    paths: Vec<PathBuf>,
}

impl ExtensionLoader {
    pub fn load_extensions(&self) -> Vec<Extension>
    pub fn reload(&self) -> Result<Vec<Extension>, Error>
}
```

**步骤2: 实现扩展API**

```rust
// src/extensions/api.rs
pub struct ExtensionAPI {
    session: AgentSession,
    ui: UIContext,
}

impl ExtensionAPI {
    pub fn register_tool(&self, tool: ToolDefinition)
    pub fn register_command(&self, name: &str, handler: CommandHandler)
    pub fn on_event(&self, event: &str, handler: EventHandler)
    pub fn send_message(&self, content: &str)
}
```

**步骤3: 提交**

```bash
git add -A && git commit -m "feat: add extension system"
```

---

## 阶段4: CLI 应用

### Task 4.1: CLI 主入口

**文件:**
- 创建: `pi/Cargo.toml`
- 创建: `pi/src/main.rs`
- 创建: `pi/src/args.rs`

**步骤1: 创建CLI项目**

```bash
cargo new pi
# 添加依赖: clap, pi-core, pi-tui
```

**步骤2: 实现参数解析**

```rust
// src/args.rs
#[derive(Parser)]
pub struct Args {
    #[arg(default_value = "")]
    pub message: String,
    
    #[arg(short, long)]
    pub continue_session: bool,
    
    #[arg(short, long)]
    pub resume: bool,
    
    #[arg(long)]
    pub session: Option<String>,
    
    #[arg(long)]
    pub provider: Option<String>,
    
    #[arg(long)]
    pub model: Option<String>,
    
    #[arg(long)]
    pub thinking: Option<String>,
    
    // ... 更多参数
}
```

**步骤3: 实现主逻辑**

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    
    if args.list_models {
        // 列出模型
    } else if args.interactive {
        // 启动TUI
    } else {
        // 单次执行
    }
}
```

**步骤4: 提交**

```bash
git add -A && git commit -m "feat: add CLI"
```

---

## 阶段5: 完善功能

### Task 5.1: Skills 系统

**文件:**
- 创建: `pi-core/src/skills/mod.rs`
- 创建: `pi-core/src/skills/loader.rs`

### Task 5.2: Prompt 模板

**文件:**
- 创建: `pi-core/src/prompts/mod.rs`
- 创建: `pi-core/src/prompts/loader.rs`

### Task 5.3: 上下文压缩

**文件:**
- 创建: `pi-core/src/compaction/mod.rs`
- 创建: `pi-core/src/compaction/summarizer.rs`

### Task 5.4: 更多的 Provider

- Azure OpenAI
- Amazon Bedrock
- Mistral
- Groq
- 等等

---

## 总结

这是一个大型工程，包含5个主要阶段:

1. **核心库 (Core Library)**: 类型、会话、工具、Provider、设置、认证
2. **Agent Core**: Agent循环、事件处理、压缩
3. **TUI 应用**: 完整的交互式终端UI
4. **CLI 应用**: 命令行入口
5. **完善功能**: Skills、Prompt模板、压缩、更多Provider

**预计时间:** 数周 (取决于投入的资源)

**建议:** 分阶段实现，每阶段单独测试和部署。
