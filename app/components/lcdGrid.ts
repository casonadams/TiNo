import { signal } from "nsh";

/** The modeled screen's width in pixels: the calculator's LCD is 320 by 240. */
const GRID_WIDTH = 320;

/** Sizes on the 320-pixel grid. The text size fits about 26 characters to a line, and a text line is 21 pixels tall, as on the calculator. */
export const GRID = {
  text: 16,
  line: 21,
  small: 12,
  status: 16,
  statusText: 10,
  inset: 6,
} as const;

const unit = signal(1);

/** Scales the grid to an LCD `width` points wide. */
export function fitGrid(width: number): void {
  if (width > 0) unit.set(width / GRID_WIDTH);
}

/** `pixels` on the 320-pixel grid, in points (tracked where it is read). */
export const px = (pixels: number): number => pixels * unit();
