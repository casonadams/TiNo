import { Text, View } from "nsh";
import type { CursorStyle, FunctionRow, FunctionsScreen } from "tino/native";
import { EntryLine } from "../EntryLine";
import { LcdRows } from "../LcdRows";
import { GRID, px } from "../lcdGrid";

function Row(props: { row: () => FunctionRow; cursorStyle: () => CursorStyle }) {
  const text = () => ({ fontSize: px(GRID.text) });
  return (
    <View class="flex-row items-center" style={{ minHeight: px(GRID.line) }} testID={`function-${props.row().label}`}>
      <Text class="font-mono text-lcd-foreground" style={{ ...text(), width: px(GRID.text * 1.3) }}>
        {props.row().label}
      </Text>
      <View
        class={`border ${props.row().onSelector ? "border-lcd-accent" : "border-transparent"} ${
          props.row().enabled ? "bg-lcd-foreground" : ""
        }`}
      >
        <Text class={`font-mono ${props.row().enabled ? "text-lcd" : "text-lcd-foreground"}`} style={text()}>
          =
        </Text>
      </View>
      <EntryLine
        entry={() => props.row().entry}
        cursor={() => props.row().cursor}
        cursorStyle={props.cursorStyle}
        size={px(GRID.text)}
      />
    </View>
  );
}

/** Y=: each function's name, its `=` filled while it is graphed, and its entry; scrolled to the focused function. */
export function FunctionsView(props: { functions: () => FunctionsScreen; cursorStyle: () => CursorStyle }) {
  const focus = () => props.functions().rows.findIndex((row) => row.onSelector || row.cursor != null);
  return (
    <LcdRows items={() => props.functions().rows} focus={focus} key={(row: FunctionRow) => row.label}>
      {(row) => <Row row={row} cursorStyle={props.cursorStyle} />}
    </LcdRows>
  );
}
