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
- **MODE**: Normal, Sci, Eng; Float or Fix 0–9; Radian or Degree; Real, a+bi or re^θi.
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
- **STAT**: 1:Edit… shows L₁–L₆ as columns: type a value and ENTER to store it (the row after the last adds one), DEL
  removes an element. CALC 1-Var Stats [list] reports x̄, Σx, Σx², Sx, σx, n and the five-number summary;
  LinReg(ax+b) [xlist, ylist] fits a line with r² and r. Lists left out are L₁ and L₂.
- **Matrices**: `[[1,2][3,4]]` (2nd × and 2nd − for the brackets), [A]–[J] from 2nd MATRIX (`→[A]` stores,
  `[A](2,1)` reads an element); `+ − ×`, a number times a matrix, `²`, `³`, `^n`, `⁻¹` and `=`/`≠`; MATRIX MATH det(,
  ᵀ, dim(, identity(, rref(. MATRIX EDIT fills a matrix cell by cell; entering past the last row or column grows it.
- **Complex numbers**: i (2nd .) in any mode (`(3+4i)(1−2i)` is `11−2i`); in a+bi and re^θi a nonreal answer
  (`√(⁻4)`, `ln(⁻1)`) is complex instead of `ERR:NONREAL ANSWER`. MATH CPX conj(, real(, imag(, angle(, abs(, ►Rect,
  ►Polar. Lists, matrices and variables A–θ hold real numbers only; Ans keeps a complex result.
- **Programs (PRGM)**: NEW names a program (ALPHA locked), EDIT opens it line by line (▲ ▼, ENTER adds a line, DEL
  on an empty line removes it), and PRGM in the editor types CTL If, Then, Else, For(, While, Repeat, End, Pause,
  prgm, Stop and I/O Input, Prompt, Disp, ClrHome; `:` separates instructions on a line. EXEC types `prgmNAME`;
  ENTER runs it, waiting at Input, Prompt and Pause, and ON breaks it. Errors Goto the program line.
- **Kept between visits**: memory, settings, functions, window, table and FORMAT settings, history, programs and the
  skin, in the browser's local storage.
- **Settings (ON)**: ON opens a settings screen for the skin, Color (the default) or Classic, and key haptics, On
  (the default) or Off; tap an option or use ▲ ▼ ◀ ▶, then ON, CLEAR or ENTER to return. Haptics vibrate on Android
  browsers and use Safari's switch-control feedback on iPhones (iOS 18+); desktop browsers have none.

Not yet: STAT PLOT, DRAW, DISTR, ANGLE, APPS, CATALOG, MEM, RCL, LINK, the ALPHA F1–F5 shortcuts and sequence
graphing (u, v, w). Their keys do nothing; syntax that needs them shows `ERR:UNSUPPORTED`.

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
