import { Text, View } from "nsh";
import type { ModeRow, ModeScreen } from "tino/native";

function Option(props: { label: string; chosen: boolean; highlighted: boolean }) {
  return (
    <View
      class={`rounded-sm border-2 px-1.5 ${props.highlighted ? "border-lcd-accent" : "border-lcd"} ${
        props.chosen ? "bg-lcd-foreground" : "bg-lcd"
      }`}
    >
      <Text class={props.chosen ? "text-lcd" : "text-lcd-foreground"} style={{ fontSize: 16 }}>
        {props.label}
      </Text>
    </View>
  );
}

function Row(props: { row: () => ModeRow }) {
  return (
    <View class="flex-row flex-wrap gap-1">
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

/** The MODE screen: a row per setting, the chosen option filled, the cursor outlined. */
export function ModeView(props: { mode: () => ModeScreen }) {
  return (
    <View class="flex-1 gap-3" testID="mode">
      <Each of={props.mode().rows} key={(row: ModeRow) => row.options[0]}>
        {(row) => <Row row={row} />}
      </Each>
    </View>
  );
}
