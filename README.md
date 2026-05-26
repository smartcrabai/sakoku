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

### Inline suppression

Add a `sakoku-ignore-next-line` marker anywhere on a line to suppress violations on the **next line** only:

```rust
// sakoku-ignore-next-line
let label = "エラー"; // non-ASCII allowed on this line only
let other = "問題";   // still flagged
```

The marker is detected as a plain string — it does not need to be inside a comment. The marker line itself is not suppressed; only the immediately following line is.

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

All other bytes (including DEL `0x7F`, null `0x00`) are flagged as violations.
At the default level — i.e. without `--strict` — multi-byte UTF-8 characters
covered by the [default allowlist](#default-unicode-allowlist) are accepted
silently as well.

## Default Unicode allowlist

Since v0.2.0, `sakoku` ships with a curated allowlist of common, low-risk
Unicode characters that are silently accepted. This makes `sakoku` usable on
modern source trees that incidentally contain accented names, prompt arrows,
emoji in tests, etc., without sprinkling `sakoku-ignore-next-line` everywhere.

| Category | Range / examples |
|---|---|
| Typography | `— – … ‘ ’ “ ” • · § ¶ © ® ™ † ‡ ‣ ‹ ›` |
| Math / comparison | `≈ ≠ ≤ ≥ ± × ÷ ∞ √ ∑ ∆ Δ π µ` |
| Arrows | full Unicode Arrows block `U+2190–U+21FF` (← → ↑ ↓ ↔ ⇐ ⇒ ↺ ↻ …) + `❮ ❯ ❰ ❱` |
| Media controls | `U+23E9–U+23FA` (⏩ ⏪ ⏸ ⏹ ⏺ …) |
| Latin script | upper Latin-1 `U+00A1–U+024F` (skips NBSP) + Latin Extended Additional `U+1E00–U+1EFF` (`naïve`, `café`, `Zürich`, `São`, `chōonpu`, `Nguyễn`, Spanish `¿¡`, French `« »`) |
| Box drawing + shapes | `U+2500–U+25FF` (─ │ ┌ ┘ ○ ● ▶ █ ▒ …) |
| Misc symbols + Dingbats | `U+2600–U+27BF` (✓ ✔ ✘ ★ ☆ ⚠ ❤ …) |
| Misc symbols + Arrows | `U+2B00–U+2BFF` (⭐ ⬆ ⬇ ⬛ ⬜ ➕ …) |
| Braille Patterns | `U+2800–U+28FF` (CLI spinners) |
| Emoji | `U+1F300–U+1FAFF`, regional indicators `U+1F1E6–U+1F1FF`, tag chars `U+E0020–U+E007F`, ZWJ `U+200D`, variation selectors `U+FE0E` / `U+FE0F` |
| Zero-width / bidi | `U+200B–U+200F`, `U+2060`, `U+FEFF` |

Categories **not** in the default allowlist (kept as violations to catch
typos and homoglyph attacks):

- CJK, Hangul, Hiragana / Katakana
- Full-width ASCII (`U+FF00–U+FF5F`)
- Cyrillic, and most of Greek (except `Δ`, `µ`, `π`)
- `U+00A0` NO-BREAK SPACE

### Security trade-off

The default allowlist accepts some characters that are also primitives of
the [Trojan Source attack class](https://trojansource.codes/) (CVE-2021-42574):
`U+200E`/`U+200F` (LRM/RLM), the zero-width joiners (`U+200B`–`U+200D`,
`U+2060`), `U+FEFF` (BOM), and the tag-character block (`U+E0020`–`U+E007F`).
The strongest bidi-override characters (`U+202A`–`U+202E`) are **not**
allowlisted and remain detected.

For security-sensitive codebases — e.g. ones that take untrusted patches or
dependencies — wire `sakoku --strict` into pre-commit hooks or CI to flag
every non-ASCII byte regardless of category.

### Disabling the default allowlist

To restore the strict 0.1.x behavior — flag **every** non-ASCII byte —
use `--strict` (alias: `--no-default-allowlist`):

```sh
sakoku --strict src/
sakoku --no-default-allowlist src/
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No violations found |
| `1` | One or more violations found |
| `2` | Error (I/O failure, no paths specified, etc.) |

## License

Apache 2.0
