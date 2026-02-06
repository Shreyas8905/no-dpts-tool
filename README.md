# 🛡️ no-dpts-tool

<div align="center">

**A high-performance Git-integrated CLI gatekeeper that enforces code quality, security, and AI-powered reviews before every commit.**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

</div>

---

## 📋 Table of Contents

- [About](#-about)
- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [Features](#-features)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Configuration](#-configuration)
- [Commands](#-commands)
- [For Users](#-for-users)
- [For Contributors](#-for-contributors)
- [Architecture](#-architecture)
- [License](#-license)

---

## 🎯 About

**no-dpts-tool** (No Dangerous Push To Source) is a pre-commit gatekeeper built in Rust that automatically runs security scans, linting checks, and AI-powered code reviews before allowing any commit to proceed. It physically blocks `git commit` if any check fails, ensuring that only high-quality, secure code makes it into your repository.

### Why "no-dpts"?

The name stands for **"No Dangerous Push To Source"** — a reminder that every commit matters, and potentially dangerous code should never slip through.

---

## 🔥 The Problem

Modern software development faces several challenges:

1. **Security Vulnerabilities Slip Through**
   - Developers accidentally commit API keys, passwords, and secrets
   - Hardcoded credentials end up in version history forever
   - Sensitive data leaks are expensive and damaging

2. **Code Quality Inconsistency**
   - Different team members have different coding standards
   - Linting is often skipped "just this once"
   - Technical debt accumulates silently

3. **Time-Consuming Code Reviews**
   - Reviewers spend time on obvious issues
   - Subtle bugs get missed under review fatigue
   - Feedback cycles are slow

4. **Existing Solutions Fall Short**
   - Pre-commit hooks are easily bypassed
   - Separate tools for security/linting/review
   - No enforcement mechanism

---

## ✅ The Solution

**no-dpts-tool** addresses these problems with a unified, enforceable gatekeeper:

| Problem | Solution |
|---------|----------|
| Secrets in code | 🔐 20+ regex patterns detect API keys, tokens, passwords |
| Missing linting | 📏 Auto-runs language-appropriate linters (ruff, eslint, cargo fmt) |
| Slow reviews | 🤖 AI-powered instant code review via Groq/LLM |
| Bypass temptation | 🚫 Physical blocking via exit code 1 |
| Emergency needs | ⚡ One-time bypass token for true emergencies |

---

## ✨ Features

### 🔐 Security Scanner
- Detects 20+ types of secrets and sensitive data:
  - AWS Access Keys & Secret Keys
  - Google API Keys & OAuth Credentials
  - GitHub Tokens & Personal Access Tokens
  - Generic API keys and secrets
  - Hardcoded passwords
  - Private keys (RSA, EC)
  - JWT tokens
  - Slack tokens
  - Database connection strings with credentials
  - Absolute local paths (system info leakage)
  - Stripe API keys
  - NPM tokens
- Support for custom regex patterns
- Severity levels (High/Medium/Low)

### 📏 Language-Aware Linting
- Automatic linter detection based on file extension:
  - Python → `ruff check`
  - JavaScript/TypeScript → `eslint`
  - Rust → `cargo fmt --check`
- Graceful fallback if linter not installed
- Parallel execution for speed

### 🤖 AI-Powered Code Review
- Integrates with Groq API for instant AI reviews
- Uses advanced LLMs (default: llama-3.3-70b-versatile)
- Analyzes for:
  - Logic bugs and errors
  - Security vulnerabilities
  - Code smells
  - Performance issues
  - Best practice violations
- Clear PASS/REJECT verdict
- Built-in rate limiting

### ⚡ Performance
- Parallel execution of all checks via `tokio`
- Only scans staged files (not entire repo)
- Efficient regex compilation
- Minimal commit-time overhead

### 🎨 Developer Experience
- Beautiful terminal output with colors and spinners
- Clear, actionable error messages
- Progress indicators for long operations
- Configurable via TOML

---

## 📦 Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/no-dpts-tool.git
cd no-dpts-tool

# Build release binary
cargo build --release

# Add to PATH (choose one):
# Option 1: Copy to a directory in your PATH
cp target/release/no-dpts-tool /usr/local/bin/

# Option 2 (Windows): Add target/release to your PATH
# Or copy to a location already in PATH
```

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs)
- **Git** - Already installed if you're reading this
- **Groq API Key** (for AI reviews) - Get one at [console.groq.com](https://console.groq.com)

---

## 🚀 Quick Start

```bash
# 1. Navigate to your Git repository
cd your-project

# 2. Initialize no-dpts-tool
no-dpts-tool init

# 3. Add your Groq API key (create .env file)
echo "GROQ_API_KEY=your-api-key-here" > .env

# 4. Stage your changes and commit
git add .
git commit -m "Your commit message"
# Checks run automatically!
```

---

## ⚙️ Configuration

Create a `no-dpts.toml` file in your project root:

```toml
# Files to ignore during scanning (supports glob patterns)
ignored_files = [
    "*.lock",
    "*.min.js",
    "*.min.css",
    "package-lock.json",
    "yarn.lock",
    "node_modules/*",
    "dist/*",
    "build/*",
]

# Custom regex patterns for project-specific secrets
# These are in addition to the built-in patterns
custom_patterns = [
    "MY_COMPANY_TOKEN_[A-Z0-9]{32}",
    "INTERNAL_API_v[0-9]+_[a-f0-9]{64}",
]

# AI model to use for code review (Groq models)
# Options: llama-3.3-70b-versatile, mixtral-8x7b-32768, etc.
ai_model = "llama-3.3-70b-versatile"

# Rate limiting for AI API calls
[rate_limit]
requests_per_minute = 30
```

### Environment Variables

Create a `.env` file in your project root:

```env
# Required for AI reviews
GROQ_API_KEY=gsk_your_api_key_here
```

> ⚠️ **Important**: Add `.env` to your `.gitignore`!

---

## 📖 Commands

### `no-dpts-tool init`

Initialize no-dpts-tool in the current Git repository.

```bash
no-dpts-tool init
```

**What it does:**
- Verifies you're in a Git repository
- Creates/overwrites `.git/hooks/pre-commit`
- Makes the hook executable
- Creates example `no-dpts.toml` if not present

### `no-dpts-tool check`

Run all checks on staged files. This is automatically called by the pre-commit hook.

```bash
no-dpts-tool check
```

**Checks performed:**
1. 🔐 Security scan for secrets/sensitive data
2. 📏 Language-aware linting
3. 🤖 AI-powered code review

**Exit codes:**
- `0` - All checks passed
- `1` - One or more checks failed (blocks commit)

### `no-dpts-tool bypass`

Create a one-time bypass token to skip checks for the next commit.

```bash
no-dpts-tool bypass
git commit -m "Emergency fix"  # Checks will be skipped once
```

> ⚠️ **Use sparingly!** The bypass is a one-time token and is intended for true emergencies only.

---

## 👤 For Users

### Daily Workflow

1. **Write your code** as usual
2. **Stage your changes**: `git add .`
3. **Commit**: `git commit -m "message"`
4. **If checks fail**:
   - Read the error messages
   - Fix the issues
   - Stage and commit again
5. **True emergency?** Use `no-dpts-tool bypass` (use sparingly!)

### Understanding Check Results

#### Security Findings
```
🔐 Security Findings:
──────────────────────────────────────────────────────────
  HIGH [AWS Access Key ID] config.py:15 - AKIA...XYZ
  MEDIUM [Generic API Key] utils.js:42 - api_k...key"
──────────────────────────────────────────────────────────
```

**How to fix:**
- Remove hardcoded credentials
- Use environment variables instead
- Use a secrets manager

#### Linting Failures
```
🔍 Linting Failures:
──────────────────────────────────────────────────────────
  ✗ main.py (ruff)
    main.py:10:5: E501 line too long (120 > 88 characters)
──────────────────────────────────────────────────────────
```

**How to fix:**
- Run the linter with `--fix`: `ruff --fix main.py`
- Or manually fix the issues

#### AI Review Rejection
```
🤖 AI Review: REJECTED
──────────────────────────────────────────────────────────
  The code contains a potential SQL injection vulnerability
  on line 45. User input is directly concatenated into the
  query string without sanitization.
  
  Recommendation: Use parameterized queries instead.
──────────────────────────────────────────────────────────
```

**How to fix:**
- Read the AI's feedback carefully
- Address the specific issues raised
- The AI is not always right — use your judgment

### Skipping Checks (When Absolutely Necessary)

```bash
# Create bypass token
no-dpts-tool bypass

# Your next commit will skip all checks
git commit -m "Hotfix: production down"

# Future commits will be checked normally
```

### Troubleshooting

| Issue | Solution |
|-------|----------|
| "GROQ_API_KEY not set" | Create `.env` file with your API key |
| "ruff is not installed" | Install: `pip install ruff` |
| "eslint is not installed" | Install: `npm install -g eslint` |
| Hook not running | Run `no-dpts-tool init` again |
| False positive on security | Add pattern to `ignored_files` in config |

---

## 🤝 For Contributors

### Project Structure

```
no-dpts-tool/
├── Cargo.toml              # Package manifest and dependencies
├── src/
│   ├── main.rs             # CLI entry point (clap)
│   ├── commands/           # Command implementations
│   │   ├── mod.rs
│   │   ├── init.rs         # Hook installation
│   │   ├── check.rs        # Main gatekeeper logic
│   │   └── bypass.rs       # Emergency bypass
│   ├── scanner/            # Code analysis
│   │   ├── mod.rs
│   │   ├── security.rs     # Secret detection
│   │   └── linter.rs       # Language-aware linting
│   ├── ai/                 # AI integration
│   │   ├── mod.rs
│   │   └── reviewer.rs     # Groq API client
│   ├── config/             # Configuration
│   │   ├── mod.rs
│   │   └── loader.rs       # TOML parser
│   └── git/                # Git operations
│       ├── mod.rs
│       └── utils.rs        # Diff, staged files, etc.
└── no-dpts.toml            # Example config
```

### Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│                    git commit                              │
└───────────────────────────┬───────────────────────────────┘
                            │
                            ▼
┌───────────────────────────────────────────────────────────┐
│              .git/hooks/pre-commit                         │
│              (calls no-dpts-tool check)                    │
└───────────────────────────┬───────────────────────────────┘
                            │
                            ▼
┌───────────────────────────────────────────────────────────┐
│                  no-dpts-tool check                        │
├───────────────────────────────────────────────────────────┤
│  1. Check for bypass token (.git/NO_DPTS_SKIP)            │
│  2. Load configuration (no-dpts.toml)                      │
│  3. Get staged files (git diff --cached --name-only)       │
│  4. Run checks in PARALLEL:                                │
│     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │
│     │  Security   │ │   Linter    │ │     AI      │       │
│     │   Scanner   │ │   Runner    │ │   Reviewer  │       │
│     └──────┬──────┘ └──────┬──────┘ └──────┬──────┘       │
│            │               │               │               │
│            ▼               ▼               ▼               │
│     ┌─────────────────────────────────────────────┐       │
│     │            Aggregate Results                 │       │
│     └─────────────────────────────────────────────┘       │
│                            │                               │
│            ┌───────────────┴───────────────┐              │
│            ▼                               ▼              │
│     ┌─────────────┐                 ┌─────────────┐       │
│     │  All Pass   │                 │  Any Fail   │       │
│     │  exit(0)    │                 │  exit(1)    │       │
│     └─────────────┘                 └─────────────┘       │
└───────────────────────────────────────────────────────────┘
```

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` v4 | CLI argument parsing with derive macros |
| `tokio` | Async runtime for parallel execution |
| `reqwest` | HTTP client for Groq API |
| `colored` | Terminal colors |
| `indicatif` | Progress bars and spinners |
| `regex` | Pattern matching for secrets |
| `serde` + `toml` | Configuration parsing |
| `governor` | Rate limiting |
| `anyhow` | Error handling |

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/no-dpts-tool.git
cd no-dpts-tool

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build in development mode
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- check

# Build release binary
cargo build --release
```

### Adding New Security Patterns

Edit `src/scanner/security.rs`:

```rust
fn get_builtin_patterns() -> Vec<(&'static str, &'static str, Severity)> {
    vec![
        // ... existing patterns ...
        
        // Add your new pattern:
        ("My New Pattern", r"PATTERN_REGEX_HERE", Severity::High),
    ]
}
```

### Adding New Linter Support

Edit `src/scanner/linter.rs`:

```rust
fn get_linter_config() -> HashMap<&'static str, (&'static str, Vec<&'static str>)> {
    let mut config = HashMap::new();
    
    // ... existing linters ...
    
    // Add new linter:
    config.insert("go", ("golint", vec![]));
    
    config
}
```

### Contributing Guidelines

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes
4. **Test** thoroughly: `cargo test`
5. **Lint** your code: `cargo fmt && cargo clippy`
6. **Commit** with conventional commits: `feat: add amazing feature`
7. **Push** to your fork
8. **Open** a Pull Request

### Code Style

- Follow Rust idioms and best practices
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write doc comments for public APIs
- Add tests for new functionality

---

## 📐 Architecture

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `main.rs` | CLI setup, command routing |
| `commands/init.rs` | Hook installation, config setup |
| `commands/check.rs` | Orchestrates all checks, aggregates results |
| `commands/bypass.rs` | Creates one-time skip token |
| `scanner/security.rs` | Regex-based secret detection |
| `scanner/linter.rs` | Runs external linters |
| `ai/reviewer.rs` | Groq API integration |
| `config/loader.rs` | TOML configuration parsing |
| `git/utils.rs` | Git command wrappers |

### Data Flow

1. **Pre-commit Hook** → Triggers `no-dpts-tool check`
2. **Check Command** → Loads config, gets staged files
3. **Parallel Execution** → Security + Linting + AI Review run concurrently
4. **Result Aggregation** → Collects all findings
5. **Exit Code** → 0 (pass) or 1 (fail)

### Error Handling

- Uses `anyhow` for ergonomic error propagation
- Graceful degradation (missing linters → skip with warning)
- User-friendly error messages with context

---

## 📜 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with ❤️ in Rust
- AI reviews powered by [Groq](https://groq.com)
- Inspired by the need for better code quality enforcement

---

<div align="center">

**[⬆ Back to Top](#-no-dpts-tool)**

*Stop pushing dangerous code. Start shipping with confidence.*

</div>
