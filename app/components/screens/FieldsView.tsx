import { Text, View } from "nsh";
import type { CursorStyle, FieldRow, FieldsScreen } from "tino/native";
import { EntryLine } from "../EntryLine";

const ROW_SIZE = 17;

function Row(props: { row: () => FieldRow; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-row items-center gap-1">
      <Text class="text-lcd-foreground" style={{ fontSize: ROW_SIZE, minWidth: 52 }}>
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
            testID={`field-${props.row().label}`}
          />
        )}
      </If>
    </View>
  );
}

/** A settings screen (WINDOW, TBLSET): each value; the focused one shows what is typed into it. */
export function FieldsView(props: { fields: () => FieldsScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1 gap-0.5" style={{ overflow: "hidden" }}>
      <Text class="text-lcd-foreground font-semibold" style={{ fontSize: 15 }}>
        {props.fields().title}
      </Text>
      <Each of={props.fields().rows} key={(row: FieldRow) => row.label}>
        {(row) => <Row row={row} cursorStyle={props.cursorStyle} />}
      </Each>
    </View>
  );
}
