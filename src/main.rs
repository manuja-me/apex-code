mod config;
mod agent;
mod tools;
mod providers;
mod tui;

use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;
use config::ApexConfig;
use agent::engine::{AgentEngine, AgentEvent};

#[derive(Parser, Debug)]
#[command(name = "apex")]
#[command(author = "manuja-me")]
#[command(version = "0.1.0")]
#[command(about = "Apex: High-performance autonomous coding agent and Swiss-style TUI", long_about = None)]
struct Cli {
    /// Initial task or prompt to run directly in the terminal
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    /// Launch interactive Swiss-style TUI
    #[arg(short, long)]
    interactive: bool,

    /// Specify model to use (overrides config)
    #[arg(short, long)]
    model: Option<String>,

    /// Target project directory (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a .apex/config.toml template in the current project
    Init,
    /// Display current Apex configuration and model fallback pool
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = ApexConfig::load()?;

    if let Some(m) = cli.model {
        config.models.primary = m;
    }

    match cli.command {
        Some(Commands::Init) => {
            let target_file = Path::new(".apex").join("config.toml");
            if target_file.exists() {
                println!("{}", "[-] .apex/config.toml already exists.".yellow());
            } else {
                ApexConfig::save_default(&target_file)?;
                println!("{}", "[+] Created .apex/config.toml with OpenRouter free model defaults.".green());
                println!("Edit .apex/config.toml to customize model pool and keys.");
            }
            return Ok(());
        }
        Some(Commands::Config) => {
            print_config_summary(&config);
            return Ok(());
        }
        None => {}
    }

    // Interactive TUI mode if requested or if no direct prompt was given
    if cli.interactive || cli.prompt.is_none() {
        println!("{}", "[+] Launching Apex Swiss TUI...".bright_cyan());
        tui::run_tui(&cli.workspace, config).await?;
        return Ok(());
    }

    // Direct Headless CLI stream mode
    if let Some(prompt) = cli.prompt {
        run_headless_cli(&cli.workspace, config, &prompt).await?;
    }

    Ok(())
}

async fn run_headless_cli(workspace: &Path, config: ApexConfig, prompt: &str) -> Result<()> {
    println!("{}", "┌─────────────────────────────────────────────────────────────┐".bright_black());
    println!(
        "│ {} {} │",
        "+ APEX//CLI".bold().white(),
        format!("[Router: {}]", config.models.primary).cyan()
    );
    println!("{}", "└─────────────────────────────────────────────────────────────┘".bright_black());
    println!("{} {}\n", "PROMPT >".bold().red(), prompt);

    let mut engine = AgentEngine::new(workspace, config)?;

    engine.run(prompt, |event| {
        match event {
            AgentEvent::ModelSelected(model) => {
                println!("{}", format!("● [Active Model: {}]", model).dimmed());
            }
            AgentEvent::AssistantMessage(msg) => {
                println!("\n{}", msg);
            }
            AgentEvent::ToolExecuting { name, args, .. } => {
                println!("{} {}: {}", "▶ TOOL.EXEC".yellow().bold(), name.bold(), args.dimmed());
            }
            AgentEvent::ToolCompleted { name, output, duration_ms, .. } => {
                println!("{} {} ({}ms)", "✔ DONE".green().bold(), name, duration_ms);
                for line in output.lines().take(5) {
                    println!("  {} {}", "│".bright_black(), line.dimmed());
                }
                if output.lines().count() > 5 {
                    println!("  {} ... (truncated)", "│".bright_black());
                }
                println!();
            }
            AgentEvent::Error(err) => {
                eprintln!("{} {}", "✖ ERROR:".red().bold(), err);
            }
            AgentEvent::UsageUpdate { session_tokens, estimated_cost, .. } => {
                println!("{}", format!("● [Usage: {} tokens | ${:.4}]", session_tokens, estimated_cost).dimmed());
            }
            AgentEvent::StepDone => {}
        }
    }).await?;

    Ok(())
}

fn print_config_summary(config: &ApexConfig) {
    println!("{}", "=== Apex Agent Configuration ===".bold().white());
    println!("Provider:        {}", config.provider.provider_type.cyan());
    println!("Base URL:        {}", config.provider.base_url);
    println!(
        "API Key:         {}",
        if config.provider.api_key.is_some() { "[SET]".green() } else { "[NOT SET - using OPENROUTER_API_KEY env]".yellow() }
    );
    println!("Primary Model:   {}", config.models.primary.bright_green());
    println!("Fast Tier Model: {}", config.models.fast_tier.cyan());
    println!("Fallback Pool:");
    for (i, m) in config.models.fallback_pool.iter().enumerate() {
        println!("  {}. {}", i + 1, m);
    }
    println!("Auto Fallback:   {}", config.models.auto_fallback);
    println!("Max Steps:       {}", config.agent.max_steps);
}
