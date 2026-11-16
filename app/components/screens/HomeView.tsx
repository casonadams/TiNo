import { Text, View } from "nsh";
import { TextView } from "nsh/base";
import type { CursorStyle, HistoryLine, HomeScreen } from "tino/native";
import { type LcdCell, lcdCells } from "../lcdCells";

function Cell(props: { cell: () => LcdCell }) {
  return (
    <View class="flex-row items-center" style={{ height: 30 }}>
      <If when={props.cell().cursor === "bar"}>
        <View class="bg-lcd-cursor" style={{ width: 2, height: 24 }} />
      </If>
      <View
        class={props.cell().cursor === "block" ? "bg-lcd-cursor rounded-sm" : "bg-lcd"}
        style={{ minWidth: props.cell().text === "" ? 12 : 0, height: 26, justifyContent: "center" }}
      >
        <Text
          class={props.cell().cursor === "block" ? "text-lcd" : "text-lcd-foreground"}
          style={{ fontSize: 22 }}
        >
          {props.cell().text}
        </Text>
      </View>
    </View>
  );
}

/** An evaluated entry: typeset on the left, its result on the right below it. */
function HistoryRow(props: { line: () => HistoryLine }) {
  return (
    <View class="gap-0.5">
      <TextView markdown={`$${props.line().latex}$`} class="text-lg text-lcd-foreground" />
      <Text class="text-lcd-foreground self-end" style={{ fontSize: 22 }}>
        {props.line().result}
      </Text>
    </View>
  );
}

/** The home screen: the newest history at the bottom, then the entry line. */
export function HomeView(props: { home: () => HomeScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1 justify-end gap-2" style={{ overflow: "hidden" }}>
      <Each of={props.home().history} key={(line: HistoryLine) => line.id}>
        {(line) => <HistoryRow line={line} />}
      </Each>
      <View class="flex-row flex-wrap items-center" testID="entry" accessibilityLabel={props.home().entry.join("")}>
        <Each of={lcdCells(props.home(), props.cursorStyle())} key={(cell: LcdCell) => cell.index}>
          {(cell) => <Cell cell={cell} />}
        </Each>
      </View>
    </View>
  );
}
