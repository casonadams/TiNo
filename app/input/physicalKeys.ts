import { keymap } from "nsh";

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

/** Binds the keyboard to `press` for as long as the calling component lives. */
export function bindPhysicalKeys(press: (key: string) => void): void {
  const bindings = Object.fromEntries(
    Object.entries(KEYSTROKES).map(([keystroke, key]) => [keystroke, () => press(key)]),
  );
  keymap.bind(bindings, { repeat: true });
}
