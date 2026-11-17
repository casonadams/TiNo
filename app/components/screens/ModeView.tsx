import { Text, View } from "nsh";
import type { ModeRow, ModeScreen } from "tino/native";
import { LcdRows } from "../LcdRows";
import { GRID, px } from "../lcdGrid";

function Option(props: { label: string; chosen: boolean; highlighted: boolean }) {
  return (
    <View
      class={`border ${props.highlighted ? "border-lcd-accent" : "border-transparent"} ${
        props.chosen ? "bg-lcd-foreground" : ""
      }`}
      style={{ paddingHorizontal: px(2) }}
    >
      <Text
        class={`font-mono ${props.chosen ? "text-lcd" : "text-lcd-foreground"}`}
        style={{ fontSize: px(GRID.text) }}
      >
        {props.label}
      </Text>
    </View>
  );
}

function Row(props: { row: () => ModeRow }) {
  return (
    <View class="flex-row flex-wrap items-center" style={{ minHeight: px(GRID.line), columnGap: px(4) }}>
      <Each of={props.row().options}>
        {(label, index) => (
          <Option
            label={label()}
            chosen={index() === props.row().selected}
            highlighted={index() === props.row().highlighted}
          />
        )}
      </Each>
    </View>
  );
}

/** The MODE screen: a line per setting, the chosen option filled, the cursor outlined; scrolled to the cursor's line. */
export function ModeView(props: { mode: () => ModeScreen }) {
  return (
    <View class="flex-1" testID="mode">
      <LcdRows
        items={() => props.mode().rows}
        focus={() => props.mode().rows.findIndex((row) => row.highlighted != null)}
        key={(row: ModeRow) => row.options[0]}
      >
        {(row) => <Row row={row} />}
      </LcdRows>
    </View>
  );
}
