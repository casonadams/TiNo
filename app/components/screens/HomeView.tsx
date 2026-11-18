import { Text, View } from "nsh";
import { TextView } from "nsh/base";
import type { CursorStyle, HistoryLine, HomeScreen } from "tino/native";
import { EntryLine } from "../EntryLine";
import { GRID, px } from "../lcdGrid";

/** A highlighted half of a history line: filled, as the calculator inverts it. */
const highlight = (selected: boolean) => (selected ? "bg-lcd-accent" : "");

/** Typeset formulas take their size from a text class, not a font size: the class nearest `points`. */
function typesetSize(points: number): string {
  const sizes: [number, string][] = [
    [14, "text-sm"],
    [16, "text-base"],
    [18, "text-lg"],
    [20, "text-xl"],
    [24, "text-2xl"],
  ];
  return sizes.reduce((best, size) => (Math.abs(size[0] - points) < Math.abs(best[0] - points) ? size : best))[1];
}

/** An evaluated entry: typeset on the left, its result on the right on the next line; the half ▲ ▼ highlight is filled. */
function HistoryRow(props: { line: () => HistoryLine }) {
  return (
    <View>
      <TextView
        markdown={`$${props.line().latex}$`}
        class={`${typesetSize(px(GRID.text))} ${highlight(props.line().entrySelected)} ${
          props.line().entrySelected ? "text-lcd" : "text-lcd-foreground"
        }`}
      />
      <If
        when={props.line().lines.length > 0}
        else={
          <View
            class={`self-end justify-center ${highlight(props.line().resultSelected)}`}
            style={{ height: px(GRID.line) }}
          >
            <Text
              class={`font-mono ${props.line().resultSelected ? "text-lcd" : "text-lcd-foreground"}`}
              style={{ fontSize: px(GRID.text) }}
            >
              {props.line().result}
            </Text>
          </View>
        }
      >
        <Each of={props.line().lines} key="index">
          {(line) => (
            <View class="justify-center" style={{ height: px(GRID.line) }}>
              <Text class="font-mono text-lcd-foreground" style={{ fontSize: px(GRID.text) }}>
                {line()}
              </Text>
            </View>
          )}
        </Each>
      </If>
    </View>
  );
}

/** The home screen: the newest history at the bottom, then the entry line. */
export function HomeView(props: { home: () => HomeScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1 justify-end" style={{ overflow: "hidden" }}>
      <Each of={props.home().history} key={(line: HistoryLine) => line.id}>
        {(line) => <HistoryRow line={line} />}
      </Each>
      <View class="flex-row">
        <EntryLine
          entry={() => props.home().entry}
          cursor={() => props.home().cursor}
          cursorStyle={props.cursorStyle}
          size={px(GRID.text)}
          testID="entry"
        />
      </View>
    </View>
  );
}
