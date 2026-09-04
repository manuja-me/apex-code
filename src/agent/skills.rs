use std::path::Path;

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub slash_command: Option<&'static str>,
    pub guidelines: &'static str,
}

pub static BUILTIN_SKILLS: &[SkillDefinition] = &[
    SkillDefinition {
        name: "Architectural Planning & Scaffolding",
        description: "Decomposes complex requests into modular components, boundary contracts, and incremental implementation phases.",
        slash_command: Some("/plan"),
        guidelines: "1. Clarify goals & constraints.\n\
                     2. Define module boundaries, types, and interface contracts.\n\
                     3. Break implementation into dependency-ordered phases.\n\
                     4. Outline automated verification tests for each phase.",
    },
    SkillDefinition {
        name: "Test-Driven Development & Self-Healing",
        description: "Executes project test runners, parses failure traces, and iteratively fixes regressions before declaring done.",
        slash_command: Some("/test"),
        guidelines: "1. Locate existing test suites and fixtures.\n\
                     2. Run test suites proactively (`cargo test`, `npm test`, `pytest`).\n\
                     3. On failure, isolate the failing assertion, inspect stack trace, and surgically fix.\n\
                     4. Re-run tests to confirm resolution.",
    },
    SkillDefinition {
        name: "Diagnostic & Compiler Resolution",
        description: "Dissects compiler diagnostics (Rust borrow checker, TypeScript types, linter warnings) and resolves root causes.",
        slash_command: None,
        guidelines: "1. Read exact compiler error codes and file/line numbers.\n\
                     2. Identify type mismatches, lifetime bounds, or missing imports.\n\
                     3. Fix root cause cleanly without loose types (`any`), unwrap cascades, or warning suppressions.",
    },
    SkillDefinition {
        name: "Code Quality & Security Review",
        description: "Audits git diffs for regressions, performance bottlenecks, unhandled errors, and vulnerability vectors.",
        slash_command: Some("/review"),
        guidelines: "1. Run git diff to inspect changes.\n\
                     2. Audit for security flaws (hardcoded secrets, injection, unsafe memory operations).\n\
                     3. Ensure proper error propagation (`?` in Rust, try/catch in TS).\n\
                     4. Eliminate dead code, redundant allocations, and untested edge cases.",
    },
    SkillDefinition {
        name: "Atomic Conventional Version Control",
        description: "Generates semantic, atomic Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`) based on verified changes.",
        slash_command: Some("/commit"),
        guidelines: "1. Verify all tests and builds pass.\n\
                     2. Stage modified files atomically.\n\
                     3. Write clear Conventional Commit message: `<type>(<scope>): <concise action-oriented summary>`.\n\
                     4. Keep changes focused and non-destructive.",
    },
    SkillDefinition {
        name: "Surgical Refactoring & Minimal Disruption",
        description: "Maintains existing code integrity, indentation, and docstrings by replacing only targeted code chunks.",
        slash_command: None,
        guidelines: "1. Inspect exact symbol locations before editing.\n\
                     2. Use `edit_file` with precise search chunks.\n\
                     3. Preserve unrelated functions, comments, and public APIs.",
    },
];

/// Detects the primary test runner for a given workspace
pub fn detect_test_runner(workspace: &Path) -> Option<(&'static str, &'static str)> {
    if workspace.join("Cargo.toml").exists() {
        Some(("Cargo Test", "cargo test"))
    } else if workspace.join("package.json").exists() {
        if workspace.join("pnpm-lock.yaml").exists() {
            Some(("pnpm test", "pnpm test"))
        } else if workspace.join("yarn.lock").exists() {
            Some(("yarn test", "yarn test"))
        } else {
            Some(("npm test", "npm test"))
        }
    } else if workspace.join("pytest.ini").exists() || workspace.join("pyproject.toml").exists() {
        Some(("Pytest", "pytest"))
    } else if workspace.join("go.mod").exists() {
        Some(("Go Test", "go test ./..."))
    } else {
        None
    }
}

/// Detects the primary linter / compiler check tool
pub fn detect_linter(workspace: &Path) -> Option<(&'static str, &'static str)> {
    if workspace.join("Cargo.toml").exists() {
        Some(("Cargo Check", "cargo check"))
    } else if workspace.join("package.json").exists() {
        Some(("TypeScript Check", "npx tsc --noEmit"))
    } else if workspace.join("pyproject.toml").exists() {
        Some(("Ruff Check", "ruff check ."))
    } else if workspace.join("go.mod").exists() {
        Some(("Go Vet", "go vet ./..."))
    } else {
        None
    }
}

/// Formats the skills reference for display in the TUI or terminal
pub fn render_skills_summary() -> String {
    let mut out = String::from("=== APEX BAKED-IN ENGINEERING SKILLS ===\n\n");
    for (i, skill) in BUILTIN_SKILLS.iter().enumerate() {
        let cmd_str = match skill.slash_command {
            Some(cmd) => format!(" [{}]", cmd),
            None => String::new(),
        };
        out.push_str(&format!("{}. {}{}\n", i + 1, skill.name, cmd_str));
        out.push_str(&format!("   {}\n\n", skill.description));
    }
    out.push_str("Use /plan <feature>, /test, /review, or /commit directly in the prompt!\n");
    out
}

/// Renders system prompt guidelines incorporating all baked-in skills
pub fn skills_system_prompt() -> String {
    let mut out = String::from("BAKED-IN ENGINEERING SKILLS & METHODOLOGIES:\n");
    for skill in BUILTIN_SKILLS {
        out.push_str(&format!("\n[SKILL: {}]\n{}\n", skill.name, skill.guidelines));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_summary() {
        let summary = render_skills_summary();
        assert!(summary.contains("Architectural Planning"));
        assert!(summary.contains("/plan"));
        assert!(summary.contains("/test"));
        assert!(summary.contains("/review"));
        assert!(summary.contains("/commit"));
    }

    #[test]
    fn test_detect_cargo_test_runner() {
        let current_dir = Path::new(".");
        let runner = detect_test_runner(current_dir);
        assert!(runner.is_some());
        let (name, cmd) = runner.unwrap();
        assert_eq!(name, "Cargo Test");
        assert_eq!(cmd, "cargo test");
    }

    #[test]
    fn test_skills_prompt_not_empty() {
        let prompt = skills_system_prompt();
        assert!(prompt.contains("BAKED-IN ENGINEERING SKILLS"));
        assert!(prompt.contains("[SKILL: Architectural Planning & Scaffolding]"));
    }
}

