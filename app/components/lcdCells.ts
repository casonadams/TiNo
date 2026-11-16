import type { CursorStyle } from "tino/native";

/** How a cell shows the cursor: not at all, as a block over it, or as a bar before it. */
export type CellCursor = "none" | "block" | "bar";

/** One drawn cell of an entry line: a token, or the empty place after the last one. */
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

/** An entry line's cells; with a cursor, the cursor on the cell it is at. */
export function lcdCells(entry: string[], cursor: number | null | undefined, style: CursorStyle): LcdCell[] {
  const shown: CellCursor = style === "insert" ? "bar" : "block";
  const cells = entry.map((text, index): LcdCell => ({
    index,
    text,
    cursor: index === cursor ? shown : "none",
  }));
  if (cursor === entry.length) {
    cells.push({ index: cells.length, text: END_GLYPHS[style], cursor: shown });
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
