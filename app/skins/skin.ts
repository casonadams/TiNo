/** How a key is drawn; each variant has its own theme tokens. */
export type KeyVariant = "default" | "function" | "number" | "operator" | "second" | "alpha";

/** One key: the core's key id and the legends printed on and above it. */
export interface KeySpec {
  key: string;
  label: string;
  second?: string;
  alpha?: string;
  variant?: KeyVariant;
}

/** A keypad: the function row, the modifier block beside the arrows, then the main grid. */
export interface KeypadLayout {
  functionRow: KeySpec[];
  control: KeySpec[][];
  dpad: { up: KeySpec; left: KeySpec; right: KeySpec; down: KeySpec };
  main: KeySpec[][];
}

/** Every key of a layout, in reading order. */
export function keysOf(layout: KeypadLayout): KeySpec[] {
  const { up, left, right, down } = layout.dpad;
  return [...layout.functionRow, ...layout.control.flat(), up, left, right, down, ...layout.main.flat()];
}
