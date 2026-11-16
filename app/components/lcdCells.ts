import type { CursorStyle, HomeScreen } from "tino/native";

/** How a cell shows the cursor: not at all, as a block over it, or as a bar before it. */
export type CellCursor = "none" | "block" | "bar";

/** One drawn cell of the entry line: a token, or the empty place after the last one. */
export interface LcdCell {
  index: number;
  text: string;
  cursor: CellCursor;
}

const END_GLYPHS: Record<CursorStyle, string> = {
  overwrite: "",
  insert: "",
  second: "↑",
  alpha: "A",
  alphaLock: "A",
};

/** The entry line's cells, with the cursor on the one it is at. */
export function lcdCells(home: HomeScreen, style: CursorStyle): LcdCell[] {
  const cursor: CellCursor = style === "insert" ? "bar" : "block";
  const cells = home.entry.map((text, index): LcdCell => ({
    index,
    text,
    cursor: index === home.cursor ? cursor : "none",
  }));
  if (home.cursor === home.entry.length) {
    cells.push({ index: cells.length, text: END_GLYPHS[style], cursor });
  }
  return cells;
}

const INDICATORS: Record<CursorStyle, string> = {
  overwrite: "",
  insert: "INS",
  second: "2ND",
  alpha: "ALPHA",
  alphaLock: "A-LOCK",
};

/** The status bar's note of the pending modifier or insert mode. */
export function modifierIndicator(style: CursorStyle): string {
  return INDICATORS[style];
}
