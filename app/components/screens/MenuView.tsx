import { Text, View } from "nsh";
import type { MenuItem, MenuScreen } from "tino/native";
import { LcdRows, type RowEdge } from "../LcdRows";
import { GRID, px } from "../lcdGrid";

const SEPARATORS: Record<"above" | "below" | "none", string> = { above: "↑", below: "↓", none: ":" };

/** A menu as the calculator draws it: the tab titles with the shown one filled, then one item per line, the highlighted item's key filled; `↑`/`↓` after a key mark items scrolled off that way. */
export function MenuView(props: { menu: () => MenuScreen }) {
  const text = () => ({ fontSize: px(GRID.text) });
  const line = () => ({ height: px(GRID.line) });
  const selected = (index: number) => index === props.menu().highlighted;
  const separator = (edge: RowEdge) => SEPARATORS[edge ?? "none"];
  return (
    <View class="flex-1" testID="menu">
      <View class="flex-row items-center" style={{ ...line(), columnGap: px(10) }}>
        <Each of={props.menu().tabs} key={(tab: string) => tab}>
          {(tab, index) => (
            <View class={index() === props.menu().tab ? "bg-lcd-foreground" : ""} style={{ paddingHorizontal: px(2) }}>
              <Text
                class={`font-mono ${index() === props.menu().tab ? "text-lcd" : "text-lcd-foreground"}`}
                style={text()}
              >
                {tab()}
              </Text>
            </View>
          )}
        </Each>
      </View>
      <LcdRows items={() => props.menu().items} focus={() => props.menu().highlighted} key={(item: MenuItem) => item.key}>
        {(item, index, edge) => (
          <View class="flex-row items-center" style={line()} accessibilityLabel={`${item().key}:${item().label}`}>
            <View class={selected(index()) ? "bg-lcd-foreground" : ""}>
              <Text class={`font-mono ${selected(index()) ? "text-lcd" : "text-lcd-foreground"}`} style={text()}>
                {`${item().key}${separator(edge())}`}
              </Text>
            </View>
            <Text class="font-mono text-lcd-foreground" style={text()}>
              {item().label}
            </Text>
          </View>
        )}
      </LcdRows>
    </View>
  );
}
