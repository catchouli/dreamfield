# Dreamfield (working title)

A PS1-style first-person dungeon crawler adventure game, an homage to the King's Field games.

## Communication
- Talk concisely and clearly. This is a game development project; keep responses short and to the point.

## Git workflow
- Never push to `main` or `develop`. Only interact with the repository via PRs.

## Coding standards
Derived from the existing codebase.

### Formatting
- Rust 2021 edition. No rustfmt/clippy configs; 4-space indent.
- `else` goes on its own line after the closing brace (`}\nelse {`).
- When a fn signature wraps to multiple lines, put the opening brace on its own line.
- Aim for a ~100 character line limit; a few pre-existing exceptions exist.
- Parenthesize compound expressions when assigning them (e.g. `x = (a == b);`).

### Comments & docs
- `///` doc comments on most items, including consts and private fns. Short, descriptive phrases.
- Prefer named constants over magic numbers, each with a doc comment. File-level consts at the top of the file; fn-local consts inside functions.
- Prefix intentionally-unused consts with `_` (e.g. `_VILLAGE_ENTRANCE`).
- Comments explain why, not what. Use `// TODO:` for known deficiencies.

### Naming
- PascalCase types, snake_case fns/fields, SCREAMING_SNAKE_CASE consts.
- Conventional fn names: `init_*`, `enter_*`, `update_*`, `*_update`, `set_*` setters.

### Imports
Order (blank lines between groups): std → external crates → workspace crates → `crate::`/`super::`.

### Architecture
- bevy_ecs (0.8): plain-fn systems with `Res`/`ResMut`/`Query`; state-driven via `SystemSet::on_enter`/`on_update` with `AppState`.
- Components live in `components.rs`, resources in `resources.rs`, per crate.
- Crate layout (path deps, not a cargo workspace): `dreamfield_system`, `dreamfield_renderer`, `dreamfield_macros`, `dreamfield_traits`.
- Module layout: `foo.rs` + `foo/bar.rs` (2018 style, no `mod.rs`).

### Error handling & safety
- `unwrap()`/`expect()` for known-safe cases; `panic!` for broken invariants.
- `log` crate macros (`log::info!`, `log::error!`, `log::debug!`) in engine code; `println!` acceptable in build scripts/proc macros.
- GL calls go in tight `unsafe` blocks; GL resources implement `Drop` (with a `log::debug!`).

### Tests
- No test suite currently; verification is by running the game.
