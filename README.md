# sakoku

A fast CLI tool to detect non-ASCII bytes in source files.

## Installation

### Homebrew (macOS / Linux)

```sh
brew install smartcrabai/homebrew-tap/sakoku
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

### GitHub Action

Run sakoku in CI using the bundled composite action. It downloads a prebuilt binary from GitHub Releases and runs sakoku over the given paths, failing the job if non-ASCII bytes are found.

```yaml
name: sakoku
on: [pull_request]
jobs:
  sakoku:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: smartcrabai/sakoku@v0.2.4
```

Pass inputs with `with:` to customize the check:

```yaml
      - uses: smartcrabai/sakoku@v0.2.4
        with:
          paths: src
          strict: 'true'
```

| Input | Description | Default |
|-------|-------------|---------|
| `paths` | Space-separated files or directories to check. | `.` |
| `strict` | Set to `true` to disable the default Unicode allowlist (runs with `--strict`). | `false` |
| `format` | Output format passed to `--format` (`compact` or `gcc`). Leave empty to use the tool default. | `''` |
| `version` | sakoku release tag to install (e.g. `v0.2.3`). | `latest` |

sakoku exits with code 1 when it detects non-ASCII bytes, which fails the job. The action supports Linux (x64/arm64), macOS (Apple Silicon / arm64), and Windows (x64/arm64) runners; macOS Intel (x64) is not supported.

Pin the action to a release tag as shown above (v0.2.4 is the first release that ships with the action), or use `smartcrabai/sakoku@main` to track the latest.

## Output format

Since v0.3.0, the default output is a **compact**, per-file digest instead of
one line per violating character. This is the format meant to be handed
straight to a coding agent — see
[Passing output to a coding agent](#passing-output-to-a-coding-agent) below.

```
$ sakoku tests/fixtures/
5 files, 31 chars
tests/fixtures/cjk_still_detected.txt [cjk,fullwidth] (4)
tests/fixtures/dirty.txt:1 [cjk] (3)
tests/fixtures/ignored.txt:3 [cjk] (1)
tests/fixtures/multi_category.txt [cjk,fullwidth,homoglyph] (15)
tests/fixtures/sparse_lines.txt [cjk] (8)
```

The first line is `{N} file(s), {M} char(s)` (singular/plural handled
correctly, e.g. `1 file, 1 char`). Each following line is:

```
{path}[:{line}] [{categories}] ({count})
```

- `path` is printed exactly as given or discovered — never rewritten to an
  absolute or relative form — so it can be passed straight back into a tool
  call that reads or edits the file.
- `line` is the 1-based line number, shown **only when every violation in
  that file falls on a single line** — as soon as a file's violations span
  two or more lines, the line number is omitted entirely (`ignored.txt:3`
  above vs. `multi_category.txt` with no `:line`). This is deliberate: when
  violations are scattered across a file, a coding agent ends up reading
  the whole file to fix it anyway, so a list of line numbers would just be
  extra output that never gets used. A single-line violation, on the other
  hand, can be fixed (or read, via `Read`'s `offset`/`limit`) without
  opening the rest of the file, so that's the one case worth the extra
  digits.
- `categories` is the sorted, deduplicated set of categories found in that
  file (see [Categories](#categories) below).
- `count` is the total number of non-ASCII characters found in that file.

If there are no violations, sakoku prints **nothing at all** — not even the
summary line.

Filtering with `--only` can turn a multi-line file into a single-line one,
at which point the line number reappears:

```
$ sakoku tests/fixtures/multi_category.txt
tests/fixtures/multi_category.txt [cjk,fullwidth,homoglyph] (15)
$ sakoku --only cjk tests/fixtures/multi_category.txt
tests/fixtures/multi_category.txt:2 [cjk] (7)
```

### Categories

Violations are classified by **remediation strategy** — what a reader should
do about the character — rather than by which Unicode block it lives in:

| Category | What it covers | How to handle it |
|---|---|---|
| `cjk` | CJK / Hangul / Hiragana / Katakana natural-language text | Needs translation — a coding agent's job |
| `fullwidth` | Full-width ASCII (`Ａ-Ｚ`, `１-９`, etc.), ideographic space (`U+3000`), NBSP (`U+00A0`) | Deterministic, mechanical substitution |
| `homoglyph` | Cyrillic / Greek look-alikes | Possible homoglyph attack — human security review, not translation |
| `symbol` | Characters on the [default allowlist](#default-unicode-allowlist) | Only reported under `--strict` |
| `other` | Everything else, including undecodable bytes | Case-by-case |

`homoglyph` is deliberately kept separate from `cjk`: if a Cyrillic or Greek
look-alike ended up in the same bucket as CJK prose, a coding agent working
through the `cjk` list might "translate" it like natural-language text and
silently launder a homoglyph attack instead of flagging it for a human.

### The pre-0.3 format (`--format gcc`)

Pass `--format gcc` to get the original GCC/Clang-compatible format back,
one line per violating character:

```
$ sakoku --format gcc tests/fixtures/dirty.txt
tests/fixtures/dirty.txt:1:10: non-ASCII byte 0xE3 ('あ')
tests/fixtures/dirty.txt:1:13: non-ASCII byte 0xE3 ('い')
tests/fixtures/dirty.txt:1:16: non-ASCII byte 0xE3 ('う')
```

Each line is `path:line:col: non-ASCII byte 0xHH` with an optional
`('char')` suffix when the byte is the start of a valid UTF-8 character.

### Filtering and limiting output

- `--only <cat>[,<cat>...]` — only report the given categories, e.g.
  `--only cjk` or `--only cjk,fullwidth`. **Comma-separated only** —
  space-separated values are intentionally not accepted, since a variadic
  option would swallow the trailing path arguments (`--only cjk fullwidth
  src/` would try to check `fullwidth` and `src/` as paths). Repeat the flag
  instead if you prefer: `--only cjk --only fullwidth`. If the filter leaves
  zero violations, sakoku exits `0` — that means "no violations in the
  requested categories", not an error.
- `--max-files <N>` — cap how many files are listed (both formats); files
  beyond the cap are summarized as `... and K more files` rather than
  silently dropped.

This limit is unlimited by default, and makes truncation visible instead of
silently losing information:

```
$ sakoku --max-files 1 tests/fixtures/
5 files, 31 chars
tests/fixtures/cjk_still_detected.txt [cjk,fullwidth] (4)
... and 4 more files
```

## Passing output to a coding agent

This is the reason the compact format exists. On a real-world sample (44
Japanese Markdown files under `~/.claude/skills`), the pre-0.3 default
produced 31,684 lines / 3.2 MB (roughly 809,000 tokens) — too large to fit
in a model's context window. The compact default produces 45 lines / 3,660
bytes (roughly 900 tokens) for the *same* violations — about a 700x
reduction by line count and an 875x reduction by byte count.

Some patterns for wiring sakoku into an agent workflow:

- **Hand only the translation work to the agent**, and nothing else:

  ```sh
  sakoku --only cjk .
  ```

  Every remaining line is a self-contained translation task: a path, a line
  number when the violation is pinpointed to one line, and how many
  characters to expect — pass the output directly into the agent's prompt.

- **Route homoglyphs to a human instead of the agent:**

  ```sh
  sakoku --only homoglyph .
  ```

  Cyrillic/Greek look-alikes are a security question, not a translation
  job — don't let an agent "fix" them by treating them as prose.

- **Fan a large tree out across parallel agent workers.** sakoku's output is
  already a file-level work list, one line per file, so it can be turned
  into a plain path list and split across N workers:

  ```sh
  sakoku --only cjk . | tail -n +2 | cut -d: -f1
  ```

- **Exclude files whose non-ASCII content is intentional** (translated
  docs, fixtures, test data) with [`.sakokuignore`](#sakokuignore) instead
  of making the agent look at them every run. This repository does exactly
  that: its own [`.sakokuignore`](.sakokuignore) excludes `README.md`,
  because this document deliberately contains non-ASCII examples like
  `('あ')`.

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
