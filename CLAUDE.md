# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust command-line text shortening utility called "shorten". It takes text input and shortens it based on configurable abbreviations and stop-word removal.

## Common Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build optimized release version

### Testing
- `cargo test` - Run all tests
- `cargo test shortener::tests::test_shorten` - Run specific test function

### Running
- `cargo run <max_length>` - Run the shortener with desired maximum length
- Example: `echo "Architecture Section Learning Session" | cargo run 10`

### Development
- `cargo check` - Quick compile check without building binaries
- `cargo clippy` - Run linter (if available)

## Architecture

### Core Components

1. **main.rs** - CLI entry point that:
   - Parses command line arguments for maximum length
   - Reads from stdin line by line
   - Outputs shortened text

2. **shortener.rs** - Main text shortening logic:
   - `Shortener` struct manages the shortening process
   - Handles word-by-word abbreviation with context awareness
   - Supports enclosed text (parentheses, brackets, quotes) preservation
   - Uses XDG directories for config file lookup at `~/.config/shorten/abbrev.lst`

3. **abbrev.rs** - Abbreviation system:
   - `Abbreviator` loads and manages abbreviation rules
   - Supports both exact text matching and regex patterns
   - Handles case preservation and attachment rules
   - Format: `Original Text = abbrev` or `Original Text = <+abbrev` (attach to previous)
   - Regex format: `/pattern/flags = replacement`

### Key Features

- **Context-aware abbreviation**: Attempts to abbreviate word pairs before individual words
- **Case preservation**: Maintains original capitalization in abbreviations
- **Enclosed text handling**: Preserves formatting for text in parentheses, brackets, quotes, etc.
- **Stop word removal**: Uses `stop-words.txt` for common words to remove
- **Configurable**: Users can provide custom abbreviation files

### Dependencies

- `eyre`/`color-eyre` - Error handling
- `itertools` - Iterator utilities
- `regex` - Pattern matching for abbreviations
- `tap` - Method chaining utilities
- `xdg` - Cross-platform config directory handling

### Test Structure

Tests are embedded in each module using `#[cfg(test)]`. The main test in `shortener.rs` demonstrates the full abbreviation workflow with sample data.