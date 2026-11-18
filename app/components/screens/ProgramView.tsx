import { Text, View } from "nsh";
import type { CursorStyle, ProgramLine, ProgramScreen } from "tino/native";
import { EntryLine } from "../EntryLine";
import { LcdRows } from "../LcdRows";
import { GRID, px } from "../lcdGrid";

/** The program editor: `PROGRAM:NAME`, then a `:` line per program line, scrolled to the line being edited. */
export function ProgramView(props: { program: () => ProgramScreen; cursorStyle: () => CursorStyle }) {
  const text = () => ({ fontSize: px(GRID.text) });
  const focus = () => props.program().lines.findIndex((line) => line.cursor != null);
  return (
    <View class="flex-1">
      <Text class="font-mono text-lcd-foreground" style={text()}>
        {props.program().title}
      </Text>
      <LcdRows items={() => props.program().lines} focus={focus} key={(line: ProgramLine) => line.id}>
        {(line) => (
          <View class="flex-row items-center" style={{ minHeight: px(GRID.line) }}>
            <Text class="font-mono text-lcd-foreground" style={text()}>
              :
            </Text>
            <EntryLine
              entry={() => line().entry}
              cursor={() => line().cursor}
              cursorStyle={props.cursorStyle}
              size={px(GRID.text)}
            />
          </View>
        )}
      </LcdRows>
    </View>
  );
}
