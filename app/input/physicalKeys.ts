import { keymap } from "nsh";
import type { CursorStyle } from "tino/native";

/** Keystrokes of a computer keyboard (as `keymap` spells them) and the calculator keys they press. */
const KEYSTROKES: Record<string, string> = {
  "0": "0",
  "1": "1",
  "2": "2",
  "3": "3",
  "4": "4",
  "5": "5",
  "6": "6",
  "7": "7",
  "8": "8",
  "9": "9",
  ".": "decimal",
  ",": "comma",
  "shift+=": "add",
  minus: "subtract",
  "shift+8": "multiply",
  "/": "divide",
  "shift+6": "power",
  "shift+9": "left-paren",
  "shift+0": "right-paren",
  enter: "enter",
  backspace: "del",
  delete: "del",
  escape: "clear",
  up: "up",
  down: "down",
  left: "left",
  right: "right",
};

/** Keystrokes that type what ALPHA and a calculator key type: the letters, space, `"`, `:` and `?`. */
const ALPHA_KEYSTROKES: Record<string, string> = {
  a: "math",
  b: "apps",
  c: "prgm",
  d: "inverse",
  e: "sin",
  f: "cos",
  g: "tan",
  h: "power",
  i: "square",
  j: "comma",
  k: "left-paren",
  l: "right-paren",
  m: "divide",
  n: "log",
  o: "7",
  p: "8",
  q: "9",
  r: "multiply",
  s: "ln",
  t: "4",
  u: "5",
  v: "6",
  w: "subtract",
  x: "store",
  y: "1",
  z: "2",
  space: "0",
  "shift+'": "add",
  "shift+;": "decimal",
  "shift+/": "negate",
};

/** Binds the keyboard to `press` for as long as the calling component lives; a letter, with Shift or without, presses ALPHA first unless ALPHA is already on (`cursorStyle` says), and drops a pending 2nd. */
export function bindPhysicalKeys(press: (key: string) => void, cursorStyle: () => CursorStyle): void {
  const typeAlpha = (key: string) => {
    const style = cursorStyle();
    if (style === "second") press("second");
    if (style !== "alpha" && style !== "alphaLock") press("alpha");
    press(key);
  };
  const alphaKeystrokes = Object.entries(ALPHA_KEYSTROKES).flatMap(([keystroke, key]) =>
    /^[a-z]$/.test(keystroke) ? [[keystroke, key], [`shift+${keystroke}`, key]] : [[keystroke, key]],
  );
  const bindings = Object.fromEntries([
    ...Object.entries(KEYSTROKES).map(([keystroke, key]) => [keystroke, () => press(key)]),
    ...alphaKeystrokes.map(([keystroke, key]) => [keystroke, () => typeAlpha(key)]),
  ]);
  keymap.bind(bindings, { repeat: true });
}
