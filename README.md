# Shorten

A Rust command-line utility for intelligently shortening text using configurable abbreviations.

## Overview

Shorten takes text input and reduces its length by applying abbreviations while preserving formatting and context. It's particularly useful for shortening long titles, descriptions, or any text that needs to fit within character limits.

## Features

- **Context-aware abbreviation**: Attempts to abbreviate word pairs before individual words
- **Case preservation**: Maintains original capitalization in abbreviations  
- **Enclosed text handling**: Preserves formatting for text in parentheses, brackets, quotes, etc.
- **Configurable abbreviations**: Support for custom abbreviation files
- **Regex patterns**: Advanced pattern matching for complex abbreviation rules
- **Attachment rules**: Control spacing and attachment behavior for abbreviations

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
git clone <repository-url>
cd shorten
cargo build --release
```

## Usage

### Basic Usage

```bash
# Shorten text to maximum 20 characters
echo "Architecture Section Learning Session" | shorten 20
# Output: Arch課 Learn Sesn

# Pipe from file
cat long-text.txt | shorten 50
```

### Configuration

Create an abbreviation file at `~/.config/shorten/abbrev.lst`:

```
# Basic abbreviations
Architecture = Arch
Learning = Learn
Session = Sesn
Department = <+部

# Regex patterns
/Meeting$/i = Mtg
/\bWeekly\b/ = W

# Remove words (empty abbreviation)
Rescheduled =

# Enclosed abbreviations
[Monthly] = [M]
[Weekly] = [W]
```

#### Abbreviation Format

- **Basic**: `Original Text = abbrev`
- **Attach to previous**: `Original Text = <+abbrev` (no space before)
- **Regex**: `/pattern/flags = replacement`
- **Remove**: `Original Text =` (empty abbreviation removes the word)

## Examples

```bash
# Input: "Architecture Section Learning Session"
# Output: "Arch課 Learn Sesn"

# Input: "*Rescheduled* [W] MPD Architecture Excellence Group Weekly Connect"  
# Output: "[W] MPD Arch Excl Group 毎週 Connect"

# Input: "[Monthly] CLSD All Hands Meeting *Rescheduled*"
# Output: "[M] CLSD All Hands Meeting"
```

## Building

```bash
# Debug build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) file for details.