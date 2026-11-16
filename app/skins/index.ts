import type { Theme } from "nsh";
import classicTheme from "./classic/theme.json";
import colorKeypad from "./color/keypad.json";
import colorTheme from "./color/theme.json";
import type { KeypadLayout } from "./skin";

/** A look for the calculator: its theme tokens and its keypad layout. */
export interface Skin {
  id: string;
  name: string;
  theme: Theme;
  keypad: KeypadLayout;
}

/** The skins to choose from; the first is the one `nsh.json` builds with. */
export const SKINS: Skin[] = [
  { id: "color", name: "Color", theme: colorTheme as Theme, keypad: colorKeypad as KeypadLayout },
  { id: "classic", name: "Classic", theme: classicTheme as Theme, keypad: colorKeypad as KeypadLayout },
];
