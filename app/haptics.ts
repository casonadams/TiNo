import { host, signal, Storage } from "nsh";
import { Haptics, isAvailable } from "nsh/mobile";

/** The storage key the haptics choice is kept under. */
const HAPTICS_KEY = "tino.haptics";

const enabled = signal(true);

/** Whether key presses play a haptic (tracked where it is read). */
export const hapticsOn = (): boolean => enabled();

/** Turns key haptics on or off and keeps the choice for the next session. */
export function setHaptics(on: boolean): void {
  enabled.set(on);
  void Storage.setItem(HAPTICS_KEY, on ? "on" : "off");
}

/** Brings back the last session's haptics choice. */
export async function restoreHaptics(): Promise<void> {
  const kept = await Storage.getItem(HAPTICS_KEY);
  if (kept !== null) enabled.set(kept === "on");
}

/** A light tap under the finger for a key press: the device's haptics in the iOS and Android apps, the page's on the web. */
export function keyTap(): void {
  if (!enabled()) return;
  if (isAvailable) Haptics.impact("light");
  else if (host.available()) host.post("haptic.tap", {});
}
