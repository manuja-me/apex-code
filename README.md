# ⚡ Apex (`apex-code`)

> **An ultra-fast, autonomous coding agent and Swiss-style TUI designed as a high-performance, cost-free alternative to Claude Code.**

[![Rust](https://img.shields.io/badge/Language-Rust%202021-DEA584.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![OmniRoute](https://img.shields.io/badge/Gateway-OmniRoute-purple.svg?style=flat-square)](https://github.com/diegosouzapw/OmniRoute)
[![TUI](https://img.shields.io/badge/UI-Swiss%20Ratatui-red.svg?style=flat-square)](https://github.com/ratatui/ratatui)

---

## Why Apex?

| Dimension | Claude Code | Apex (`apex-code`) |
| :--- | :--- | :--- |
| **Runtime & Startup** | Node.js / TS (~100MB+ RAM, slower startup) | **Rust** (<10ms startup, single static binary, ~15MB RAM) |
| **Model Lock-In** | Anthropic Claude API only (expensive) | **OmniRoute AI Gateway** (350+ providers, local-first) |
| **429 Rate Limit Handling** | Fails or blocks | **Zero-Cost Failover Pool** (rotates models automatically) |
| **Interface** | Standard scrolling CLI text | **Swiss International Typographic TUI** (`ratatui`) |
| **Search Engine** | External shell grep / glob | **Embedded ripgrep engine** (`ignore` + `regex`) |
| **Distribution** | Requires `npm` / `node` installation | **Single portable executable** (`apex.exe` / `apex`) |

---

## Swiss TUI Interface

Apex features a terminal user interface inspired by the **Swiss International Typographic Style** (Josef Müller-Brockmann, Massimo Vignelli):

```text
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│ [+] APEX//CLI  RELEASE 0.1.0   │ ROUTING: FLASH ➔ QWEN   │ 18.4K/1M TOKENS │ $0.0000 │ MAIN*   │
├──────────────────────────────────────────────────────────────┬─────────────────────────────────┤
│ // 01. USER INTENT                                           │ // 01. LSP COMPILER HEALTH      │
│ [01] Optimize auth middleware to cache claims with Redis.    │ ┌──────────────┬──────────────┐ │
│                                                              │ │ ERRORS:  00  │ WARNS:   01  │ │
│ TIER 01 // FLASH (FAST AST SCAN - 6.2ms)                     │ │ SYMBOLS: 412 │ RESP:   12MS │ │
│ Analyzed 14 translation units • Isolated 2 dependency nodes  │ └──────────────┴──────────────┘ │
│                                                              ├─────────────────────────────────┤
│ TOOL.EXEC — RIPGREP "verify_token"            [3 HITS • 0.4MS│ // 02. TOKEN DENSITY (1.8%)     │
│ ┌──────────────────────────────────────────────────────────┐ │ ■■□□□□□□□□□□□□□□□□□□□□□□□□□□□□  │
│ │ src/auth/jwt.rs:42: pub async fn verify_token(...)       │ │ IN-MEMORY: 18.4K / 1000K        │
│ └──────────────────────────────────────────────────────────┘ ├─────────────────────────────────┤
│ [DIFF] SRC/MIDDLEWARE/AUTH.RS                [+14 / -3]      │ // 03. ACTIVE ASSETS            │
│ ┌──────────────────────────────────────────────────────────┐ │ • Cargo.toml (180 tokens)       │
│ │ 18 - let claims = jwt::verify_token(&auth_header).await? │ │ • src/main.rs (420 tokens)      │
│ │ 18 + // Fast L1 memory & Redis cache lookup              │ │                                 │
│ │ 19 + if let Some(cached) = redis.get(&token).await? {    │ │ [TAB] SIDEBAR    [ESC] STOP     │
│ └──────────────────────────────────────────────────────────┘ │ [ENTER] SEND     [UP/DN] SCROLL │
├──────────────────────────────────────────────────────────────┴─────────────────────────────────┤
│ PROMPT> Write unit test for the cache miss path in tests/auth_flow.rs     [ENTER ↵] [TAB]      │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Features

### 1. Seamless OmniRoute AI Gateway Support
Apex works out of the box with the [OmniRoute](https://github.com/diegosouzapw/OmniRoute) AI Gateway (`http://localhost:20128/v1`), connecting you to 350+ AI providers:
* **Token Compression**: Integrates with OmniRoute's RTK + Caveman engines to compress prompts and tool outputs, saving 15–95% of tokens.
* **100% Free Tier Priority**: Pre-configured with the best free models (`qwen/qwen-2.5-coder-32b-instruct:free`, `deepseek/deepseek-r1:free`, `google/gemini-2.0-flash-exp:free`, etc.).
* **Zero Local Friction**: No mandatory external API key required for local instances—defaults to local authorization automatically.

### 2. Automatic Zero-Cost Model Failover
If a model hits a `429 Too Many Requests` or temporary queue saturation, **Apex automatically hot-swaps to the next model in the pool without interrupting your work session.**

### 3. Native Embedded Tooling Engine
Unlike other agents that shell out to slow external scripts, Apex embeds its core tools directly into the binary:
* **Embedded Ripgrep**: High-speed regex file searching using the `ignore` and `regex` crates (respects `.gitignore`).
* **Surgical File Editor**: Exact, unique block replacement to avoid rewriting entire files.
* **Sandboxed Command Runner**: Async shell execution with automatic timeout and output truncation.
* **Git Operations**: Real-time git status, branch detection, and unstaged diff inspections.

### 4. Dual Operational Modes
* **Interactive TUI Mode**: Run `apex` to launch the full split-screen dashboard.
* **Headless Stream Mode**: Run `apex "refactor auth middleware"` to execute tasks directly in your terminal stream.

---

## Built-In Agent Tools

| Tool Name | Description |
| :--- | :--- |
| `ripgrep` | Ultra-fast regex and text search across the workspace (respects `.gitignore`). |
| `find_files` | Fuzzy file and directory name discovery across the repository. |
| `view_file` | Reads file content with customizable line-range windows and line numbers. |
| `edit_file` | Applies surgical search-and-replace patches to existing files. |
| `write_file` | Creates new files or overwrites existing ones with directory auto-creation. |
| `run_command` | Executes terminal commands with timeout enforcement and captured output. |
| `git_status` | Returns short branch status, staged modifications, and untracked files. |
| `git_diff` | Shows unstaged git diffs across the entire project or for a specific file. |

---

## Baked-In Engineering Skills & Playbooks

Apex embeds full software engineering playbooks directly into its reasoning loop and interactive prompt interface:

| Slash Command | Skill Set | Description |
| :--- | :--- | :--- |
| `/skills` | **Skills Reference** | Lists all baked-in engineering playbooks and usage guidelines. |
| `/plan <goal>` | **Architecture & Scaffolding** | Breaks down large features into module boundaries, types, and phased implementation plans. |
| `/test [args]` | **TDD & Self-Healing** | Auto-detects workspace test runner (`cargo test`, `npm test`, `pytest`, `go test`) and executes suites. |
| `/review` | **Security & Code Quality** | Audits unstaged `git diff` against security flaws, regressions, unhandled errors, and dead code. |
| `/commit [msg]` | **Conventional Version Control** | Generates atomic Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`) from verified diffs. |
| `/model <id>` | **Live Hot-Swap** | Switches active model on the fly without restarting your session. |
| `/diff` | **Git Diff View** | Displays live project-wide unstaged modifications in the TUI stream. |
| `/status` | **Engine Telemetry** | Displays active session tokens, costs, git branch, and workspace metrics. |

---

## Quickstart

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (edition 2021+)
* [OmniRoute](https://github.com/diegosouzapw/OmniRoute) running locally (e.g. `docker run -p 20128:20128 diegosouzapw/omniroute` or `npm i -g omniroute && omniroute`)

### 1. Installation
```bash
git clone https://github.com/manuja-me/apex-code.git
cd apex-code
cargo build --release
```
The compiled binary will be located at `target/release/apex.exe` (or `target/release/apex` on Linux/macOS).

### 2. Configure OmniRoute (Optional)
Apex connects to `http://localhost:20128/v1` automatically. If your OmniRoute gateway runs on a different port or host, set:
```bash
# Windows (PowerShell)
$env:OMNIROUTE_BASE_URL = "http://localhost:20128/v1"
$env:OMNIROUTE_API_KEY = "your_key_if_configured"

# Linux / macOS
export OMNIROUTE_BASE_URL="http://localhost:20128/v1"
export OMNIROUTE_API_KEY="your_key_if_configured"
```

Or initialize a project-level configuration file:
```bash
apex init
```
This generates a `.apex/config.toml` pre-configured for OmniRoute.

### 3. Usage Examples

#### Launch the Interactive Swiss TUI:
```bash
apex
```

#### Plan an Architectural Feature:
Inside the TUI:
```text
PROMPT> /plan Add JWT auth with Redis token blacklist
```

#### Run Workspace Tests:
```text
PROMPT> /test
```

#### Audit Unstaged Diffs:
```text
PROMPT> /review
```

#### Run a Single Task Headless:
```bash
apex "search for all unwrapped error results and replace with proper ? operators"
```

#### View Current Configuration & Active Model Pool:
```bash
apex config
```

---

## Keyboard Navigation (TUI)

| Keybinding | Action |
| :--- | :--- |
| <kbd>Enter</kbd> | Submit prompt to the agent |
| <kbd>Tab</kbd> | Cycle sidebar panel (Telemetry / Workers / Skills & Controls) |
| <kbd>Up</kbd> / <kbd>Down</kbd> | Browse prompt history |
| <kbd>PageUp</kbd> / <kbd>PageDn</kbd> | Scroll conversation and execution stream |
| <kbd>Esc</kbd> | Interrupt current agent task / Exit |
| <kbd>Ctrl+C</kbd> | Force quit application |

---

## Configuration (`.apex/config.toml`)

```toml
[provider]
provider_type = "omniroute"
base_url = "http://localhost:20128/v1"
api_key = "omniroute" # Optional for local OmniRoute instances

[models]
primary = "qwen/qwen-2.5-coder-32b-instruct:free"
fallback_pool = [
    "qwen/qwen-2.5-coder-32b-instruct:free",
    "deepseek/deepseek-r1:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "google/gemini-2.0-flash-exp:free"
]
fast_tier = "google/gemini-2.0-flash-exp:free"
auto_fallback = true
max_retries = 3

[agent]
workspace_dir = "."
max_steps = 30
temperature = 0.2
max_tokens = 4096
```

---

## Branching Strategy

* **`main`**: Production-ready, stable releases.
* **`development`**: Active feature development, experimental tools, and provider integrations.

---

## License
MIT License &copy; 2026 [manuja-me](https://github.com/manuja-me)
