import { Text, View } from "nsh";
import type { CursorStyle, WindowRow, WindowScreen } from "tino/native";
import { EntryLine } from "../EntryLine";

const ROW_SIZE = 17;

function Row(props: { row: () => WindowRow; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-row items-center gap-1">
      <Text class="text-lcd-foreground" style={{ fontSize: ROW_SIZE, width: 52 }}>
        {`${props.row().label}=`}
      </Text>
      <If
        when={props.row().entry}
        else={
          <Text class="text-lcd-foreground" style={{ fontSize: ROW_SIZE }}>
            {props.row().value}
          </Text>
        }
      >
        {(entry) => (
          <EntryLine
            entry={entry}
            cursor={() => props.row().cursor}
            cursorStyle={props.cursorStyle}
            size={ROW_SIZE}
            testID={`window-${props.row().label}`}
          />
        )}
      </If>
    </View>
  );
}

/** WINDOW: each setting's value; the focused one shows what is typed into it. */
export function WindowView(props: { window: () => WindowScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1 gap-0.5" style={{ overflow: "hidden" }}>
      <Text class="text-lcd-foreground font-semibold" style={{ fontSize: 15 }}>
        WINDOW
      </Text>
      <Each of={props.window().rows} key={(row: WindowRow) => row.label}>
        {(row) => <Row row={row} cursorStyle={props.cursorStyle} />}
      </Each>
    </View>
  );
}
