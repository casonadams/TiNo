import { signal } from "nsh";
import { setHaptics } from "./haptics";
import { SKINS } from "./skins";
import { chooseSkin, currentSkin } from "./skinStore";
import { press } from "./store";

/** Keys that close the settings screen. */
const CLOSING_KEYS = ["on", "clear", "enter"];

/** The settings screen's lines. */
export type SettingsLine = "skin" | "haptics";
const LINES: SettingsLine[] = ["skin", "haptics"];

const open = signal(false);
const line = signal<SettingsLine>("skin");

/** Whether the settings screen ON opens is shown (tracked where it is read). */
export const settingsOpen = (): boolean => open();

/** The settings line the cursor is on (tracked where it is read). */
export const settingsLine = (): SettingsLine => line();

/** ◀ ▶ on a line: the previous or next skin (wrapping), or haptics on or off. */
function stepLine(by: number): void {
  if (line() === "haptics") {
    setHaptics(by < 0);
    return;
  }
  const index = SKINS.findIndex((skin) => skin.id === currentSkin().id);
  chooseSkin(SKINS[(index + by + SKINS.length) % SKINS.length].id);
}

/** Presses the key with id `key`: ON opens the settings screen, where ▲ ▼ pick a line, ◀ ▶ change it, and ON, CLEAR or ENTER close it; every other key goes to the calculator. */
export function pressKey(key: string): void {
  if (!open()) {
    if (key === "on") {
      line.set("skin");
      open.set(true);
    } else press(key);
    return;
  }
  if (key === "up" || key === "down") line.set(LINES[(LINES.indexOf(line()) + 1) % LINES.length]);
  if (key === "left") stepLine(-1);
  if (key === "right") stepLine(1);
  if (CLOSING_KEYS.includes(key)) open.set(false);
}
