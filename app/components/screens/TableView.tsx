import { Text, View } from "nsh";
import type { CursorStyle, TableScreen } from "tino/native";
import { scrollTable } from "../../store";
import { EntryLine } from "../EntryLine";
import { px } from "../lcdGrid";

/** Table text and row height on the 320-pixel grid: eight rows under a header fit the screen. */
const cellSize = () => px(13);
/** A row's height in points, also the drag distance that scrolls one row. */
const rowHeight = () => px(19);
/** Wheel distance, in points, of one line of a line-based wheel. */
const LINE_HEIGHT = 16;

/** A table cell with its position, so equal values keep distinct keys. */
interface TableCell {
  column: number;
  text: string;
}

/** A row of the table with its position, so equal rows keep distinct keys. */
interface TableLine {
  row: number;
  cells: TableCell[];
}

function Cell(props: { text: string; selected: boolean }) {
  return (
    <View
      class={`flex-1 items-center justify-center ${props.selected ? "bg-lcd-foreground" : ""}`}
      style={{ height: rowHeight() }}
    >
      <Text
        class={`font-mono ${props.selected ? "text-lcd" : "text-lcd-foreground"}`}
        style={{ fontSize: cellSize(), numberOfLines: 1 }}
      >
        {props.text}
      </Text>
    </View>
  );
}

/** The table (TABLE, or STAT EDIT's lists): a value per cell, the selected cell filled, its value or what is typed into it written below. Wheel or drag scrolls TABLE. */
export function TableView(props: { table: () => TableScreen; cursorStyle: () => CursorStyle }) {
  let dragStart = 0;
  let dragged = 0;
  let wheeled = 0;
  const lines = (): TableLine[] =>
    props.table().rows.map((cells, row) => ({ row, cells: cells.map((text, column) => ({ column, text })) }));
  return (
    <View
      class="flex-1"
      testID="table"
      onStartShouldSetResponder={() => true}
      onResponderGrant={(e) => {
        dragStart = e.pageY;
        dragged = 0;
        return true;
      }}
      onResponderMove={(e) => {
        const rows = Math.trunc((e.pageY - dragStart - dragged) / rowHeight());
        if (rows === 0) return;
        dragged += rows * rowHeight();
        scrollTable(-rows);
      }}
      onWheel={(e) => {
        wheeled += e.delta.y * (e.deltaMode === "lines" ? LINE_HEIGHT : 1);
        const rows = Math.trunc(wheeled / rowHeight());
        if (rows === 0) return;
        wheeled -= rows * rowHeight();
        scrollTable(rows);
      }}
    >
      <View class="flex-row gap-0.5 border-b border-lcd-muted">
        <Each of={props.table().headers}>{(header) => <Cell text={header()} selected={false} />}</Each>
      </View>
      <Each of={lines()} key={(line: TableLine) => line.row}>
        {(line) => (
          <View class="flex-row gap-0.5">
            <Each of={line().cells} key={(cell: TableCell) => cell.column}>
              {(cell) => (
                <Cell
                  text={cell().text}
                  selected={line().row === props.table().selectedRow && cell().column === props.table().selectedColumn}
                />
              )}
            </Each>
          </View>
        )}
      </Each>
      <View class="flex-row items-center">
        <Text class="font-mono text-lcd-foreground" style={{ fontSize: cellSize() }} testID="table-footer">
          {props.table().footer}
        </Text>
        <If when={props.table().entry}>
          {(entry) => (
            <EntryLine
              entry={entry}
              cursor={() => props.table().entryCursor}
              cursorStyle={props.cursorStyle}
              size={cellSize()}
            />
          )}
        </If>
      </View>
    </View>
  );
}
