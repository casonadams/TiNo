import { Text, View } from "nsh";
import type { Screen } from "tino/native";
import { type LcdCell, lcdCells, modifierIndicator } from "./lcdCells";

function Cell(props: { cell: () => LcdCell }) {
  return (
    <View class="flex-row items-center" style={{ height: 30 }}>
      <If when={props.cell().cursor === "bar"}>
        <View class="bg-lcd-cursor" style={{ width: 2, height: 24 }} />
      </If>
      <View
        class={props.cell().cursor === "block" ? "bg-lcd-cursor rounded-sm" : "bg-lcd"}
        style={{ minWidth: 12, height: 26, justifyContent: "center" }}
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

/** The screen: a status bar over the entry line. */
export function Lcd(props: { screen: () => Screen }) {
  return (
    <View class="bg-bezel rounded-xl p-3">
      <View class="bg-lcd rounded-md gap-2 p-3" style={{ aspectRatio: 4 / 3 }}>
        <Text class="text-lcd-muted" style={{ fontSize: 12, height: 16 }}>
          {modifierIndicator(props.screen().cursorStyle)}
        </Text>
        <View
          class="flex-row flex-wrap items-center"
          testID="entry"
          accessibilityLabel={props.screen().entry.join("")}
        >
          <Each of={lcdCells(props.screen())} key={(cell: LcdCell) => cell.index}>
            {(cell) => <Cell cell={cell} />}
          </Each>
        </View>
      </View>
    </View>
  );
}
