import { Text, View } from "nsh";
import type { CursorStyle } from "tino/native";
import { type LcdCell, lcdCells } from "./lcdCells";

/** A cell `size` points high in text, on a line 1.3 times that; the cursor block covers the glyph's height. */
function Cell(props: { cell: () => LcdCell; size: number }) {
  return (
    <View class="flex-row items-center" style={{ height: props.size * 1.3 }}>
      <If when={props.cell().cursor === "bar"}>
        <View class="bg-lcd-cursor" style={{ width: Math.max(1, props.size / 8), height: props.size * 1.15 }} />
      </If>
      <View
        class={props.cell().cursor === "block" ? "bg-lcd-cursor" : ""}
        style={{ minWidth: props.cell().text === "" ? props.size * 0.6 : 0, height: props.size * 1.2, justifyContent: "center" }}
      >
        <Text
          class={`font-mono ${props.cell().cursor === "block" ? "text-lcd" : "text-lcd-foreground"}`}
          style={{ fontSize: props.size }}
        >
          {props.cell().text}
        </Text>
      </View>
    </View>
  );
}

/**
 * A line of tokens; with a `cursor`, the cursor drawn at that token as `cursorStyle` says.
 * Its accessibility label is the line's text.
 */
export function EntryLine(props: {
  entry: () => string[];
  cursor: () => number | null | undefined;
  cursorStyle: () => CursorStyle;
  size: number;
  testID?: string;
}) {
  return (
    <View
      class="flex-1 flex-row flex-wrap items-center"
      testID={props.testID}
      accessibilityLabel={props.entry().join("")}
    >
      <Each of={lcdCells(props.entry(), props.cursor(), props.cursorStyle())} key={(cell: LcdCell) => cell.index}>
        {(cell) => <Cell cell={cell} size={props.size} />}
      </Each>
    </View>
  );
}
