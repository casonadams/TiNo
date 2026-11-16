import { Text, View } from "nsh";
import type { TableScreen } from "tino/native";
import { scrollTable } from "../../store";

const CELL_SIZE = 15;
/** Drag distance, in points, that scrolls one row. */
const ROW_HEIGHT = 20;
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

function Cell(props: { text: string; selected: boolean; header?: boolean }) {
  return (
    <View
      class={`flex-1 items-center justify-center rounded-sm ${props.selected ? "bg-lcd-foreground" : "bg-lcd"}`}
      style={{ height: ROW_HEIGHT }}
    >
      <Text
        class={`${props.selected ? "text-lcd" : "text-lcd-foreground"} ${props.header ? "font-semibold" : ""}`}
        style={{ fontSize: CELL_SIZE, numberOfLines: 1 }}
      >
        {props.text}
      </Text>
    </View>
  );
}

/** The table: X and each shown function per row, the selected cell filled, its value written below. Wheel or drag scrolls. */
export function TableView(props: { table: () => TableScreen }) {
  let dragStart = 0;
  let dragged = 0;
  let wheeled = 0;
  const lines = (): TableLine[] =>
    props.table().rows.map((cells, row) => ({ row, cells: cells.map((text, column) => ({ column, text })) }));
  return (
    <View
      class="flex-1 gap-0.5"
      testID="table"
      onStartShouldSetResponder={() => true}
      onResponderGrant={(e) => {
        dragStart = e.pageY;
        dragged = 0;
        return true;
      }}
      onResponderMove={(e) => {
        const rows = Math.trunc((e.pageY - dragStart - dragged) / ROW_HEIGHT);
        if (rows === 0) return;
        dragged += rows * ROW_HEIGHT;
        scrollTable(-rows);
      }}
      onWheel={(e) => {
        wheeled += e.delta.y * (e.deltaMode === "lines" ? LINE_HEIGHT : 1);
        const rows = Math.trunc(wheeled / ROW_HEIGHT);
        if (rows === 0) return;
        wheeled -= rows * ROW_HEIGHT;
        scrollTable(rows);
      }}
    >
      <View class="flex-row gap-0.5 border-b border-lcd-muted">
        <Each of={props.table().headers}>{(header) => <Cell text={header()} selected={false} header />}</Each>
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
      <Text class="text-lcd-foreground" style={{ fontSize: 13 }} testID="table-footer">
        {props.table().footer}
      </Text>
    </View>
  );
}
