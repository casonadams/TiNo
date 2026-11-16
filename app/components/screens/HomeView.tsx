import { Text, View } from "nsh";
import { TextView } from "nsh/base";
import type { CursorStyle, HistoryLine, HomeScreen } from "tino/native";
import { EntryLine } from "../EntryLine";

/** An evaluated entry: typeset on the left, its result on the right below it; the half ▲ ▼ highlight is outlined. */
function HistoryRow(props: { line: () => HistoryLine }) {
  const outline = (selected: boolean) => `rounded-sm border-2 px-0.5 ${selected ? "border-lcd-accent" : "border-lcd"}`;
  return (
    <View class="gap-0.5">
      <View class={`self-start ${outline(props.line().entrySelected)}`}>
        <TextView markdown={`$${props.line().latex}$`} class="text-lg text-lcd-foreground" />
      </View>
      <View class={`self-end ${outline(props.line().resultSelected)}`}>
        <Text class="text-lcd-foreground" style={{ fontSize: 22 }}>
          {props.line().result}
        </Text>
      </View>
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
      <View class="flex-row">
        <EntryLine
          entry={() => props.home().entry}
          cursor={() => props.home().cursor}
          cursorStyle={props.cursorStyle}
          size={22}
          testID="entry"
        />
      </View>
    </View>
  );
}
