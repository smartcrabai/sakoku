# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

### Changed

- **Breaking: the default output format changed from GCC/Clang-style
  (one line per violating character) to a compact, per-file digest**
  (one line per file: `path[:line] [categories] (count)`). The line
  number is included only when every violation in the file sits on a
  single line; once violations span two or more lines, the number is
  left out, because a caller that has to touch more than one line ends
  up reading the whole file anyway, making a line-range list dead
  weight. The point of the change is to make sakoku's output cheap to
  hand to a coding agent: on a real-world sample (44 Japanese Markdown
  files), the old default produced 31,684 lines / 3.2 MB (roughly
  809,000 tokens), while the new compact default produces 45 lines /
  3,660 bytes (roughly 900 tokens) for the same violations -- about a
  700x reduction by line count and an 875x reduction by byte count. The
  old format did not fit in a model's context window at all; the new
  one does, comfortably.
  - To restore the pre-0.3 behavior exactly, pass `--format gcc`.
  - If a violation-free run produced no output before, it still does:
    compact mode prints nothing at all when there are zero violations.
  - **Exit code semantics are unchanged**: `0` = no violations, `1` =
    violations found, `2` = error. Scripts and CI steps that only check
    the exit code (as the bundled GitHub Action does) are unaffected by
    this change.

### Added

- `--format <compact|gcc>` flag to select the output format explicitly.
  `compact` is the new default; `gcc` reproduces the pre-0.3 one-line-
  per-character format.
- Violation categories, classified by remediation strategy rather than
  Unicode block: `cjk` (natural-language CJK/Hangul text that needs
  translation), `fullwidth` (full-width ASCII, ideographic space, NBSP
  -- mechanically replaceable), `homoglyph` (Cyrillic/Greek -- possible
  homoglyph attack, needs human security review), `symbol` (characters
  on the default allowlist, only reported under `--strict`), and
  `other` (everything else). `homoglyph` is deliberately kept separate
  from `cjk` so a coding agent handed a `cjk` list doesn't try to
  "translate" a Cyrillic homoglyph and accidentally paper over an
  attack.
- `--only <cat>[,<cat>...]` flag to filter reported violations down to
  one or more categories, e.g. `--only cjk` or `--only cjk,fullwidth`.
  Comma-separated only -- space-separated values are intentionally not
  supported, since a variadic option would swallow the trailing path
  arguments. If the filter leaves zero violations, the run exits `0`
  ("no violations in the requested categories", not an error).
- `--max-files <N>` flag to cap how many files are listed (both output
  formats); files beyond the cap are summarized as
  `... and K more files` rather than silently dropped.

## [0.2.0]

### Added

- **Default Unicode allowlist.** A curated set of common, low-risk Unicode
  characters is now silently accepted by default. Modern source trees that
  contain accented names (`naïve`, `café`, `chōonpu`, `Nguyễn`), Spanish
  punctuation (`¿`, `¡`), French/German guillemets (`« »`), prompt arrows
  (`❯`), math comparison (`≈ ≠ ≤ ≥ Δ`), typography (`—`, `…`), ZWJ emoji
  sequences (`👨‍👩‍👧`), regional-indicator flags (`🇯🇵`), README badges
  (`⭐`, `⬆`), and box drawing / spinner characters no longer need per-line
  `sakoku-ignore-next-line` suppression.
- `--strict` flag (alias: `--no-default-allowlist`) restores the original
  0.1.x behavior — every non-ASCII byte is reported.
- `CheckOptions` struct on the library side, threaded through `check_bytes`
  and `walk_and_check`.
- Crate-root re-exports for `CheckOptions`, `Violation`, `check_bytes`,
  `is_default_allowed`, `FileResult`, `walk_and_check`, `format_violation`,
  and `SakokuError` — downstream library users no longer need to reach
  through `sakoku::checker::*` etc.

### Changed

- **Breaking (library):** `check_bytes` now takes `CheckOptions` as a second
  argument. CLI users are unaffected. Library callers should pass
  `CheckOptions::default()` to opt into the new behavior, or
  `CheckOptions { strict: true }` for the old behavior.
- **Breaking (library):** `walk_and_check` similarly gains a `CheckOptions`
  parameter.
- `is_allowed_unicode` is removed and replaced by the broader
  `is_default_allowed`, which covers the full default allowlist (the old
  characters remain a subset).

### Notes

CJK, Hangul, Hiragana/Katakana, full-width ASCII, Cyrillic, most of Greek
(except `Δ`, `µ`, `π`), and `U+00A0` NO-BREAK SPACE are deliberately **not**
in the default allowlist — they remain detected to catch typos and homoglyph
attacks. Projects that intentionally use these scripts can either keep the
existing `sakoku-ignore-next-line` markers or list them in a `.sakokuignore`
file.

### Security trade-off

The default allowlist accepts a few characters that are also primitives of
the "Trojan Source" attack class (CVE-2021-42574): `U+200E` (LRM),
`U+200F` (RLM), `U+200B`–`U+200D` / `U+2060` (zero-width joiners),
`U+FEFF` (BOM), and `U+E0020`–`U+E007F` (tag characters). They are useful
in legitimate code (NFC handling, emoji ZWJ sequences, flag tag sequences)
and were explicitly requested in the spec, but a determined attacker can
use them to smuggle invisible text into source.

The strong bidi-override characters that drive the original Trojan Source
PoC — `U+202A`–`U+202E` — remain **detected** at the default level.

Projects that need a stricter posture should run with `--strict`
(or `--no-default-allowlist`), which restores the 0.1.x policy of flagging
every non-ASCII byte. Consider wiring `sakoku --strict` into pre-commit
hooks or CI for security-sensitive codebases.

## [0.1.5]

Last release with the strict-only behavior. From 0.2.0 onward, run `sakoku`
with `--strict` to reproduce 0.1.x output.
