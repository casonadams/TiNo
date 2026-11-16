# AGENTS.md

Guidance for coding agents and contributors working in this repository.

## Architecture

Three layers; dependencies point down only.

| Layer | Path | Owns | Must not |
|---|---|---|---|
| Domain | `crates/calc-core` | Keys, keymap, tokens, editor, parser, evaluator, formatting, MODE settings, every screen's state machine, graphing, table, save/load | Depend on `nsh-*` or do I/O |
| Bridge | `app/native` | Converting calc-core types to JS values (`JsObject`, `PropEnum`, the `#[class] Calculator`) | Hold behavior; it only converts |
| App | `app/*.ts(x)` | Drawing the screen model, the keypad from skin data, gestures, keyboard, storage I/O, skins | Decide calculator behavior |

- `Calculator::press(Key)` is the single entry for key presses; `Calculator::screen()` is the single view model.
  Gestures enter through `pan_graph`, `zoom_graph`, `trace_at` and `scroll_table`, all taking fractions, never pixels.
- Each screen has a state type (`ModeCursor`, `ErrorMenu`, `YCursor`, `FieldCursor<F>`, `TableCursor`,
  `TraceCursor`, `ListMenu`) and a handler in `crates/calc-core/src/calculator/`. The screen keys (Y=, WINDOW, ZOOM,
  TRACE, GRAPH, TBLSET, TABLE, MODE, 2nd QUIT) are routed in `calculator/mod.rs` and work from every screen but the
  error screen.
- Storage is the app's job: `app/store.ts` saves `Calculator.save()` under `tino.calculator` after every change and
  restores it on start; `app/skinStore.ts` keeps the skin under `tino.skin`. Rust's `nsh_core::storage` is not
  reachable from class methods called by TSX.

## Commands

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && nsh check && nsh test
```

All five must pass before a commit. CI (`.github/workflows/ci.yml`) runs them, packages the web build and deploys
`main` to GitHub Pages. A local web build needs `CC_wasm32_unknown_unknown` and `AR_wasm32_unknown_unknown` pointing
at a WebAssembly clang and `llvm-ar` (see README).

## Quality gates

- `clippy.toml`: cognitive complexity at most 5, at most 3 arguments (including `self`), functions at most 60 lines.
  Split by responsibility, bundle related arguments into a type (`ZoomContext`, `Scale`, `GraphPoint`), or move the
  title or constants onto a trait (`Field::TITLE`). Do not add `#[allow]`.
- No dead code, placeholders or stubs. Keys whose behavior is not built yet map to a `Command` that the screens
  ignore; syntax that parses but is not evaluated returns `ErrorKind::Unsupported`.

## Conventions

- **Calculator behavior follows the TI-84 Plus CE** unless a decision below says otherwise: precedence, error
  names (`ERR:SYNTAX`), menu numbering (ZOOM's ZStandard is 6), formatting (10 significant digits, `.5`, `⁻`, `ᴇ`).
  Re-describe behavior in our own words; never copy TI manual text, ROM data or fonts.
- **Naming**: no TI model names or logos in the product, skins or UI.
- **Decisions in force**: f64 numbers shown to 10 digits; modern display (typeset entries, vector graphs); Rust core
  with a TSX UI; web is the only target for now.
- **Doc comments on everything `app/native` exports** are required: NSH generates `.nsh/types` TypeScript from them
  and warns without them. Keep them short and about behavior.
- **Colors come from theme tokens only** (`bg-key-number`, `token("graph-1")`). Every skin's `theme.json` defines
  the same token set; add a token to every skin when you add one. No raw colors or palette classes in components.
- **Skins** are data: `app/skins/<id>/theme.json` and a keypad layout, registered in `app/skins/index.ts`. The
  app test "every key of every skin" catches a key id the core does not know.
- **Saved state** has a version (`SNAPSHOT_VERSION` in `calculator/persist.rs`). A new field with a serde default keeps
  the version; any change an older snapshot cannot be read under means a new version, and the app starts fresh.
- **NSH** is pinned in `mise.toml` and as `=0.5.1` for `nsh-core` in `Cargo.toml`; upgrade both together and rerun
  every check. Never edit `app/.nsh/`; it is generated.

## Tests

- Domain rules: unit tests beside the code; whole key sequences in `crates/calc-core/tests/calculator/` (one
  integration crate, shared helpers in `support.rs`). Prefer table-driven cases.
- App: `app/App.test.tsx`, headless through `nsh test`, pressing keys by `id:key-<key id>` and asserting on texts,
  labels and storage. Test behavior a user can see, not wiring.

## Commits

Conventional Commits (`feat:`, `fix:`, `style(app):`), imperative subject, a short bullet body.
