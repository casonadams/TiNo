import { setTheme, signal, Storage } from "nsh";
import { SKINS, type Skin } from "./skins";

/** The storage key the chosen skin's id is kept under. */
const SKIN_KEY = "tino.skin";

const skin = signal<Skin>(SKINS[0]);

/** The skin shown (tracked where it is read). */
export const currentSkin = (): Skin => skin();

function apply(next: Skin): void {
  setTheme(next.theme);
  skin.set(next);
}

/** Shows the skin with id `id` and keeps the choice for the next session. */
export function chooseSkin(id: string): void {
  const next = SKINS.find((candidate) => candidate.id === id);
  if (!next) throw new Error(`no skin "${id}"`);
  apply(next);
  void Storage.setItem(SKIN_KEY, id);
}

/** Shows the skin the last session chose, when it is one of `SKINS`. */
export async function restoreSkin(): Promise<void> {
  const id = await Storage.getItem(SKIN_KEY);
  const kept = SKINS.find((candidate) => candidate.id === id);
  if (kept) apply(kept);
}
