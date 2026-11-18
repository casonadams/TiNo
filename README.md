# TiNo

A graphing calculator in the style of the TI-84 Plus CE, for the browser. Live at
**https://casonadams.github.io/TiNo/**.

Built with [Native Shell](https://nsh.dev): the calculator itself is a Rust crate with no UI dependency, and a small
TSX app draws its screen and keypad. TiNo is an independent clean-room project, not affiliated with or endorsed by
Texas Instruments.

## What it does

- **Home screen**: TI-style token entry (`sin(` is one token), 2nd and ALPHA (2nd ALPHA locks), overwrite and insert
  editing, the calculator's precedence (implied multiplication, `⁻2²` is ⁻4, `^` groups left to right), Ans, `→`
  into A–Z and θ, ENTER to repeat, 2nd ENTRY to recall, ▲ ▼ to highlight an earlier entry or result and ENTER to
  type it at the cursor, typeset entries with Float/Fix/Sci/Eng results.
- **MATH, 2nd TEST and VARS menus**: ►Frac, ►Dec, ³, ³√(, ˣ√, fMin(, fMax(, nDeriv(, fnInt(, Σ(, logBASE(;
  abs(, round(, iPart(, fPart(, int(, min(, max(, lcm(, gcd(, remainder(; rand, nPr, nCr, !, randInt(; = ≠ > ≥ < ≤;
  and, or, xor, not(; the WINDOW and TBLSET settings and Y₁–Y₀ (`Y₁(3)` evaluates Y₁ at 3). ◀ ▶ switch tabs, ▲ ▼
  and ENTER or an item's key pick; they type into the home screen, Y= or a WINDOW/TBLSET field.
- **MODE**: Normal, Sci, Eng; Float or Fix 0–9; Radian or Degree.
- **Errors**: an error screen with Quit and Goto (back to the token at fault).
- **Graphing**: ten Y= functions, WINDOW, ZOOM (ZBox, Zoom In/Out, ZDecimal, ZSquare, ZStandard, ZTrig, ZoomFit),
  TRACE, a free cursor on the arrow keys, and FORMAT (2nd ZOOM: CoordOn/Off, GridOff/Dot/Line, AxesOn/Off,
  LabelOff/On, ExprOn/Off); drag to pan, wheel or pinch to zoom, tap to trace.
- **CALC (2nd TRACE)**: value, zero, minimum, maximum, intersect, dy/dx and ∫f(x)dx (shaded); mark bounds, curves
  and guesses with the trace cursor and ENTER.
- **TABLE and TBLSET**: X from TblStart by ΔTbl beside the graphed functions; wheel or drag to scroll.
- **Lists**: `{1,2,3}`, L₁–L₆ (2nd 1–6; `→L₁` stores, `L₁(2)` reads an element), math element by element (`2L₁+1`,
  `sin(L₁)`, `Y₁(L₁)`), and 2nd LIST: names; OPS dim(, seq(, cumSum(, ΔList(, augment(; MATH min(, max(, mean(,
  median(, sum(, prod(, stdDev(, variance(.
- **Kept between visits**: memory, settings, functions, window, table and FORMAT settings, history and the skin, in the
  browser's local storage.
- **Settings (ON)**: ON opens a settings screen for the skin, Color (the default) or Classic, and key haptics, On
  (the default) or Off; tap an option or use ▲ ▼ ◀ ▶, then ON, CLEAR or ENTER to return. Haptics vibrate on Android
  browsers and use Safari's switch-control feedback on iPhones (iOS 18+); desktop browsers have none.

Not yet: statistics (STAT), matrices, complex numbers and programs.
Keys for them are on the keypad and do nothing for now; syntax that needs them shows `ERR:UNSUPPORTED`.

### Computer keyboard

| Keys | Calculator key |
|---|---|
| `0`–`9` `.` `,` | digits, decimal point, comma |
| `+` `-` `*` `/` `^` `(` `)` | + − × ÷ ^ ( ) |
| Enter, Backspace or Delete, Escape | ENTER, DEL, CLEAR |
| Arrow keys | ▲ ▼ ◀ ▶ |

## Development

Tools: [mise](https://mise.jdx.dev) (pins the `nsh` CLI in `mise.toml`), Rust stable 1.97 or newer with the
`wasm32-unknown-unknown` target (`rust-toolchain.toml` adds it), and for web builds a clang that targets
WebAssembly plus `llvm-ar` (on macOS, Homebrew's `llvm`).

```sh
mise install                                    # the pinned nsh

cargo test --workspace                          # the calculator's tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cd app
nsh check                                       # type-checks the TSX against the Rust exports
nsh test                                        # the app's headless tests
nsh run --target web                            # dev server with hot reload

# a production web build (macOS)
CC_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/clang \
AR_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/llvm-ar \
  nsh package --target web                      # writes app/.nsh/out/TiNo/
```

Every push to `main` runs the checks above in GitHub Actions and deploys the web build to GitHub Pages
(`.github/workflows/ci.yml`).

## Layout

```
crates/calc-core/   the calculator: keys, tokens, parser, evaluator, formatting, screens, graphing, table,
                    save/load; no UI or NSH dependency
app/native/         the Rust bridge: exports calc-core to TSX as `tino/native`
app/                the TSX app: App.tsx, store.ts, components/, skins/
```

[AGENTS.md](AGENTS.md) describes the architecture and conventions in more detail.

## License

[MIT](LICENSE)
