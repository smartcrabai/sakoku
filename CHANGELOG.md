# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
