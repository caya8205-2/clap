# clap

Rust port of a Python mass-rename script — a CLI that renames every file
in a folder to `Name 1`, `Name 2`, etc, ordered by mtime (oldest -> newest).

Safe to run multiple times on the same folder: it does a two-phase rename
(move to a unique UUID temp name first, then to the final name), so numbers
never skip or collide, even if the final names already exist.

## Build

```bash
cargo build --release
```

Binary ends up at `target/release/clap` (or `.exe` on Windows).

## Install (global, like `npm link` for Node CLIs)

```bash
cargo install --path .
```

This puts `clap` on your `PATH` (via `~/.cargo/bin`), so you can
call it from anywhere without prefixing `./target/release/`.

## Usage

Direct mode (non-interactive, good for scripts/aliases):

```bash
clap -p ./folder -n "Photo"
```

Dry-run mode (preview the rename plan without touching anything):

```bash
clap -p ./folder -n "Photo" --dry-run
```

Interactive mode (no flags, prompts you):

```bash
clap
```