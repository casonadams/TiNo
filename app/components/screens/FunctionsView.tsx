import { Text, View } from "nsh";
import type { CursorStyle, FunctionRow, FunctionsScreen } from "tino/native";
import { EntryLine } from "../EntryLine";

const ROW_SIZE = 15;

function Row(props: { row: () => FunctionRow; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-row items-center gap-1" testID={`function-${props.row().label}`}>
      <Text class="text-lcd-foreground" style={{ fontSize: ROW_SIZE, width: 22 }}>
        {props.row().label}
      </Text>
      <View
        class={`rounded-sm border-2 px-0.5 ${props.row().onSelector ? "border-lcd-accent" : "border-lcd"} ${
          props.row().enabled ? "bg-lcd-foreground" : "bg-lcd"
        }`}
      >
        <Text class={props.row().enabled ? "text-lcd" : "text-lcd-foreground"} style={{ fontSize: ROW_SIZE }}>
          =
        </Text>
      </View>
      <EntryLine
        entry={() => props.row().entry}
        cursor={() => props.row().cursor}
        cursorStyle={props.cursorStyle}
        size={ROW_SIZE}
      />
    </View>
  );
}

/** Y=: each function's name, its `=` filled while it is graphed, and its entry. */
export function FunctionsView(props: { functions: () => FunctionsScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1" style={{ overflow: "hidden" }}>
      <Each of={props.functions().rows} key={(row: FunctionRow) => row.label}>
        {(row) => <Row row={row} cursorStyle={props.cursorStyle} />}
      </Each>
    </View>
  );
}
