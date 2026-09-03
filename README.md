# ⚡ Apex (`apex-code`)

> **An ultra-fast, autonomous coding agent and Swiss-style TUI designed as a high-performance alternative to Claude Code.**

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

## Architecture

```mermaid
flowchart TD
    subgraph UI ["Terminal User Interface"]
        TUI[Swiss-Style TUI (Ratatui)]
        CLI[Direct Headless Runner]
    end

    subgraph Core ["Apex Runtime Engine (Rust / Tokio)"]
        ReAct[ReAct Autonomous Orchestrator]
        Router[Multi-Model Router & Failover Pool]
        Context[Context & AST Scope Engine]
    end

    subgraph Providers ["Model Providers"]
        OpenRouter[OpenRouter Free Tier (/chat/completions)]
        Qwen[Qwen 2.5 Coder 32B Free]
        DeepSeek[DeepSeek R1 Free]
        Gemini[Gemini 2.0 Flash Exp Free]
        Llama[Llama 3.3 70B Free]
    end

    subgraph Tools ["Native Tooling Engine"]
        RG[Embedded Ripgrep Search]
        Files[Surgical File Reader / Editor]
        Shell[Sandboxed Terminal Exec]
        Git[Git Status & Unified Diff]
    end

    UI --> Core
    Core --> Tools
    Core --> Router
    Router --> OpenRouter
    OpenRouter --> Qwen
    OpenRouter --> DeepSeek
    OpenRouter --> Gemini
    OpenRouter --> Llama
```

---

## Key Features

### 1. 100% Free OpenRouter Tier Support
Apex works out of the box with OpenRouter's free `:free` models:
* `qwen/qwen-2.5-coder-32b-instruct:free` (Primary coding model)
* `deepseek/deepseek-r1:free` (Deep step-by-step reasoning)
* `meta-llama/llama-3.3-70b-instruct:free` (Broad refactoring)
* `google/gemini-2.0-flash-exp:free` (Lightning-fast indexing & search)

### 2. Automatic Zero-Cost Model Failover
If OpenRouter's free tier hits a `429 Too Many Requests` or queue saturation, **Apex automatically hot-swaps to the next free model in the pool without interrupting your work session.**

### 3. Swiss International Typographic TUI
Inspired by Josef Müller-Brockmann and modernist design principles:
* **8:4 Asymmetric Grid**: Clear separation between execution logs and compiler telemetry.
* **Typographic Hierarchy**: Monospace high-precision layouts with Swiss Red (`#EB0029`) accents.
* **Compiler Telemetry**: Real-time diagnostic matrices, symbol counts, and token density gauges.

### 4. Dual Operational Modes
* **Interactive TUI Mode**: Run `apex` to open the full split-screen dashboard.
* **Headless Stream Mode**: Run `apex "refactor auth middleware"` to execute single tasks directly in your current shell.

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
Set your key in your shell:
```bash
# Windows (PowerShell)
$env:OPENROUTER_API_KEY = "your_openrouter_key"

# Linux / macOS
export OPENROUTER_API_KEY="your_openrouter_key"
```

Or initialize a local configuration file in your project:
```bash
apex init
```
This generates a `.apex/config.toml` where you can permanently set your key and customize model pools.

### 3. Usage

#### Launch the Interactive Swiss TUI:
```bash
apex
```

#### Run a Single Task Headless:
```bash
apex "search for all unwrapped error results and replace with proper ? operators"
```

#### View Current Configuration:
```bash
apex config
```

---

## Configuration (`.apex/config.toml`)

```toml
[provider]
provider_type = "openrouter"
base_url = "https://openrouter.ai/api/v1"
# api_key = "sk-or-v1-..." (Optional if set in environment)

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

## License
MIT License &copy; 2026 [manuja-me](https://github.com/manuja-me)
