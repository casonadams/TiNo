import { Text, View } from "nsh";
import type { CursorStyle, FieldRow, FieldsScreen } from "tino/native";
import { EntryLine } from "../EntryLine";
import { LcdRows } from "../LcdRows";
import { GRID, px } from "../lcdGrid";

function Row(props: { row: () => FieldRow; cursorStyle: () => CursorStyle }) {
  const text = () => ({ fontSize: px(GRID.text) });
  return (
    <View class="flex-row items-center" style={{ height: px(GRID.line) }}>
      <Text class="font-mono text-lcd-foreground" style={text()}>
        {` ${props.row().label}=`}
      </Text>
      <If
        when={props.row().entry}
        else={
          <Text class="font-mono text-lcd-foreground" style={text()}>
            {props.row().value}
          </Text>
        }
      >
        {(entry) => (
          <EntryLine
            entry={entry}
            cursor={() => props.row().cursor}
            cursorStyle={props.cursorStyle}
            size={px(GRID.text)}
            testID={`field-${props.row().label}`}
          />
        )}
      </If>
    </View>
  );
}

/** A settings screen (WINDOW, TBLSET): its title, then one value per line; the focused one shows what is typed into it. */
export function FieldsView(props: { fields: () => FieldsScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1" style={{ overflow: "hidden" }}>
      <View class="justify-center" style={{ height: px(GRID.line) }}>
        <Text class="font-mono text-lcd-foreground" style={{ fontSize: px(GRID.text) }}>
          {props.fields().title}
        </Text>
      </View>
      <LcdRows
        items={() => props.fields().rows}
        focus={() => props.fields().rows.findIndex((row) => row.entry != null)}
        key={(row: FieldRow) => row.label}
      >
        {(row) => <Row row={row} cursorStyle={props.cursorStyle} />}
      </LcdRows>
    </View>
  );
}
