# rsight

<p align="center">
  <img src="landing/public/logo.png" alt="rsight logo" width="200" />
</p>

A fast terminal search tool for macOS. Run `rsight` in any directory to search file names, file contents, and AI conversations from a single interface.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **File & folder search** — fuzzy matching across the current directory
- **Content search** — search inside files with result snippets and line numbers
- **AI conversation search** — find past conversations from Claude Code and Cursor
- **Instant results** — parallel search with 150ms debounce; results stream in as you type
- **Open from results** — open files in `$EDITOR`, resume AI conversations in their native tool

## Install

**Homebrew (recommended)**

```sh
brew tap olucasandrade/rsight https://github.com/olucasandrade/rsight
brew install rsight
```

**curl**

```sh
curl -fsSL https://raw.githubusercontent.com/olucasandrade/rsight/main/install.sh | bash
```

Installs to `~/.local/bin`. Override with `INSTALL_DIR=/usr/local/bin bash <(curl ...)`.

**From source**

```sh
cargo install --git https://github.com/olucasandrade/rsight
```

## Usage

```sh
rsight
```

Run from any directory. Results update as you type across all four tabs simultaneously, scoped to the current directory.

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch tabs |
| `↑` / `↓` | Navigate results |
| `Enter` | Open selected result |
| `Ctrl+C` | Copy path to clipboard |
| `Esc` | Quit |

### Tabs

| Tab | Searches | Opens with |
|-----|----------|-----------|
| Files | File names under current directory | `$EDITOR` |
| Folders | Directory names under current directory | Finder |
| Contents | Text inside files under current directory | `$EDITOR` at matching line |
| AI | Claude Code conversations | `claude --resume` |

## AI conversation search

rsight finds conversations from:

- **Claude Code** — reads JSONL sessions from `~/.claude/projects/`

Results show the conversation title and date. Press `Enter` to resume the conversation in Claude Code.

## Performance

Benchmarks run on a 500-file corpus (Apple Silicon, optimized build):

| Search type | Time |
|-------------|------|
| Name search | ~2.5 ms |
| Content search | ~5.2 ms |

## Building from source

Requires Rust 1.70+.

```sh
git clone https://github.com/olucasandrade/rsight
cd rsight
cargo build --release
./target/release/rsight
```

Run tests:

```sh
cargo test
```

Run benchmarks:

```sh
cargo bench
```

## Contributing

Issues and pull requests are welcome. For significant changes, open an issue first to discuss the approach.

## License

MIT
