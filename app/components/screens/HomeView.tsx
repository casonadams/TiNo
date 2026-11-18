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

/** A line of text, left aligned: a report line or a program's Disp text. */
function TextLine(props: { text: () => string }) {
  return (
    <View class="justify-center" style={{ height: px(GRID.line) }}>
      <Text class="font-mono text-lcd-foreground" style={{ fontSize: px(GRID.text) }}>
        {props.text()}
      </Text>
    </View>
  );
}

/** A history line: the entry typeset on the left, report lines, and the result on the right, each part shown when it has any; the half ▲ ▼ highlight is filled. */
function HistoryRow(props: { line: () => HistoryLine }) {
  return (
    <View>
      <If when={props.line().latex !== ""}>
        <TextView
          markdown={`$${props.line().latex}$`}
          class={`${typesetSize(px(GRID.text))} ${highlight(props.line().entrySelected)} ${
            props.line().entrySelected ? "text-lcd" : "text-lcd-foreground"
          }`}
        />
      </If>
      <Each of={props.line().lines} key="index">
        {(line) => <TextLine text={line} />}
      </Each>
      <If when={props.line().result !== ""}>
        <View class={`self-end ${highlight(props.line().resultSelected)}`}>
          <Each of={props.line().result.split("\n")} key="index">
            {(row) => (
              <View class="justify-center" style={{ height: px(GRID.line) }}>
                <Text
                  class={`font-mono ${props.line().resultSelected ? "text-lcd" : "text-lcd-foreground"}`}
                  style={{ fontSize: px(GRID.text) }}
                >
                  {row()}
                </Text>
              </View>
            )}
          </Each>
        </View>
      </If>
    </View>
  );
}

/** The home screen: the newest history at the bottom, then the entry line, after a running program's prompt while it waits for input. */
export function HomeView(props: { home: () => HomeScreen; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-1 justify-end" style={{ overflow: "hidden" }}>
      <Each of={props.home().history} key={(line: HistoryLine) => line.id}>
        {(line) => <HistoryRow line={line} />}
      </Each>
      <View class="flex-row items-center">
        <If when={props.home().prompt}>
          {(prompt) => (
            <Text class="font-mono text-lcd-foreground" style={{ fontSize: px(GRID.text) }}>
              {prompt()}
            </Text>
          )}
        </If>
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
