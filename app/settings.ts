import { signal } from "nsh";
import { SKINS } from "./skins";
import { chooseSkin, currentSkin } from "./skinStore";
import { press } from "./store";

/** Keys that close the settings screen. */
const CLOSING_KEYS = ["on", "clear", "enter"];

const open = signal(false);

/** Whether the settings screen ON opens is shown (tracked where it is read). */
export const settingsOpen = (): boolean => open();

/** ▲ ▼ show the previous or next skin, wrapping. */
function stepSkin(by: number): void {
  const index = SKINS.findIndex((skin) => skin.id === currentSkin().id);
  chooseSkin(SKINS[(index + by + SKINS.length) % SKINS.length].id);
}

/** Presses the key with id `key`: ON opens the settings screen, which then takes ▲ ▼ and closes on ON, CLEAR or ENTER; every other key goes to the calculator. */
export function pressKey(key: string): void {
  if (!open()) {
    if (key === "on") open.set(true);
    else press(key);
    return;
  }
  if (key === "up") stepSkin(-1);
  if (key === "down") stepSkin(1);
  if (CLOSING_KEYS.includes(key)) open.set(false);
}
