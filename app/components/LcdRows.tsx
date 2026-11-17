import { type JSX, signal, View } from "nsh";
import { GRID, px } from "./lcdGrid";

/** Which way rows are scrolled off past a shown row: before the first shown, after the last shown, or neither. */
export type RowEdge = "above" | "below" | null;

/** A row with its place in the whole list. */
interface PlacedRow<T> {
  item: T;
  index: number;
}

/** Text lines, one grid line each, as many as fit the space: scrolled so line `focus` shows, the last shown when it lies below the first screenful, as the calculator scrolls its lists. */
export function LcdRows<T>(props: {
  items: () => readonly T[];
  focus: () => number;
  key: (item: T) => unknown;
  children: (item: () => T, index: () => number, edge: () => RowEdge) => JSX.Element;
}) {
  const fits = signal(1);
  const first = () => Math.max(0, props.focus() + 1 - fits());
  const shown = (): PlacedRow<T>[] =>
    props
      .items()
      .slice(first(), first() + fits())
      .map((item, offset) => ({ item, index: first() + offset }));
  const edge = (index: number): RowEdge => {
    if (index === first() && first() > 0) return "above";
    if (index === first() + fits() - 1 && index < props.items().length - 1) return "below";
    return null;
  };
  return (
    <View
      class="flex-1"
      style={{ overflow: "hidden" }}
      onLayout={(e) => fits.set(Math.max(1, Math.floor(e.bounds.size.height / px(GRID.line))))}
    >
      <Each of={shown()} key={(row: PlacedRow<T>) => props.key(row.item)}>
        {(row) =>
          props.children(
            () => row().item,
            () => row().index,
            () => edge(row().index),
          )
        }
      </Each>
    </View>
  );
}
