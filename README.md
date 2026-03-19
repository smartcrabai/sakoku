# sakoku

A fast CLI tool to detect non-ASCII bytes in source files.

## Installation

### Homebrew (macOS / Linux)

```sh
brew install takumi3488/homebrew-tap/sakoku
```

### Shell installer (cargo-dist)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/smartcrabai/sakoku/releases/latest/download/sakoku-installer.sh | sh
```

### cargo

```sh
cargo install sakoku
```

## Usage

### Check files or directories

```sh
sakoku src/
sakoku main.rs lib.rs
sakoku .
```

### Read from stdin

```sh
cat main.rs | sakoku --stdin
```

### `.sakokuignore`

Place a `.sakokuignore` file in any directory to exclude paths from scanning, using the same syntax as `.gitignore`. `.gitignore` files are also respected automatically. Hidden files (dotfiles) are skipped by default.

## Output format

Violations are printed in GCC/Clang-compatible format:

```
path/to/file.rs:12:5: non-ASCII byte 0xE3 ('あ')
path/to/file.rs:15:1: non-ASCII byte 0xFF
```

Each line is `path:line:col: non-ASCII byte 0xHH` with an optional `('char')` suffix when the byte is the start of a valid UTF-8 character.

## Allowed bytes

The following bytes are permitted and will not be reported:

| Byte | Description |
|------|-------------|
| `0x09` | Horizontal tab |
| `0x0A` | Line feed (LF) |
| `0x0D` | Carriage return (CR) |
| `0x20`–`0x7E` | Printable ASCII |

All other bytes (including DEL `0x7F`, null `0x00`, and any multi-byte UTF-8 sequences) are flagged as violations.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No violations found |
| `1` | One or more violations found |
| `2` | Error (I/O failure, no paths specified, etc.) |

## License

Apache 2.0
