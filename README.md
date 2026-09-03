# ⚡ Apex (`apex-code`)

> **An ultra-fast, autonomous coding agent and Swiss-style TUI designed as a high-performance, cost-free alternative to Claude Code.**

[![Rust](https://img.shields.io/badge/Language-Rust%202021-DEA584.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![OpenRouter](https://img.shields.io/badge/Provider-OpenRouter%20Free%20Tier-purple.svg?style=flat-square)](https://openrouter.ai/)
[![TUI](https://img.shields.io/badge/UI-Swiss%20Ratatui-red.svg?style=flat-square)](https://github.com/ratatui/ratatui)

---

## Why Apex?

| Dimension | Claude Code | Apex (`apex-code`) |
| :--- | :--- | :--- |
| **Runtime & Startup** | Node.js / TS (~100MB+ RAM, slower startup) | **Rust** (<10ms startup, single static binary, ~15MB RAM) |
| **Model Lock-In** | Anthropic Claude API only (expensive) | **OpenRouter Free Tier + Multi-Provider** ($0.00 cost) |
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

### 1. 100% Free OpenRouter Tier Support
Apex works out of the box with OpenRouter's free `:free` models:
* **`qwen/qwen-2.5-coder-32b-instruct:free`** &mdash; Primary coding, editing, and diff engine.
* **`deepseek/deepseek-r1:free`** &mdash; Deep step-by-step reasoning and algorithmic planning.
* **`meta-llama/llama-3.3-70b-instruct:free`** &mdash; Broad refactoring and instruction following.
* **`google/gemini-2.0-flash-exp:free`** &mdash; Lightning-fast repository search and file indexing.

### 2. Automatic Zero-Cost Model Failover
If OpenRouter's free tier encounters a `429 Too Many Requests` or temporary queue saturation, **Apex automatically hot-swaps to the next free model in the pool without interrupting your work session.**

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

## Quickstart

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (edition 2021+)
* A free [OpenRouter API Key](https://openrouter.ai/keys)

### 1. Installation
```bash
git clone https://github.com/manuja-me/apex-code.git
cd apex-code
cargo build --release
```
The compiled binary will be located at `target/release/apex.exe` (or `target/release/apex` on Linux/macOS).

### 2. Configure Your Free OpenRouter Key
Set your key in your current shell:
```bash
# Windows (PowerShell)
$env:OPENROUTER_API_KEY = "your_openrouter_key"

# Linux / macOS
export OPENROUTER_API_KEY="your_openrouter_key"
```

Or initialize a project-level configuration file:
```bash
apex init
```
This generates a `.apex/config.toml` where you can permanently configure API keys and model pools.

### 3. Usage Examples

#### Launch the Interactive Swiss TUI:
```bash
apex
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
| <kbd>Tab</kbd> | Cycle sidebar panel (Telemetry / Workers / Keymap) |
| <kbd>Up</kbd> / <kbd>Down</kbd> | Scroll conversation and execution stream |
| <kbd>Esc</kbd> | Interrupt current agent task / Exit |
| <kbd>Ctrl+C</kbd> | Force quit application |

---

## Configuration (`.apex/config.toml`)

```toml
[provider]
provider_type = "openrouter"
base_url = "https://openrouter.ai/api/v1"
# api_key = "sk-or-v1-..." (Optional if set via OPENROUTER_API_KEY)

[models]
primary = "qwen/qwen-2.5-coder-32b-instruct:free"
fallback_pool = [
    "qwen/qwen-2.5-coder-32b-instruct:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "deepseek/deepseek-r1:free",
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
