import { signal, Text, View } from "nsh";
import type { Screen } from "tino/native";
import { modifierIndicator } from "./lcdCells";
import { ErrorView } from "./screens/ErrorView";
import { FieldsView } from "./screens/FieldsView";
import { FunctionsView } from "./screens/FunctionsView";
import { GraphView } from "./screens/GraphView";
import { HomeView } from "./screens/HomeView";
import { MenuView } from "./screens/MenuView";
import { ModeView } from "./screens/ModeView";
import { SettingsView } from "./screens/SettingsView";
import { TableView } from "./screens/TableView";

/** The status bar: the settings in force, then the pending modifier. */
function StatusBar(props: { screen: () => Screen }) {
  return (
    <View class="flex-row justify-between" style={{ height: 16 }}>
      <Text class="text-lcd-muted" style={{ fontSize: 12 }}>
        {props.screen().status.join(" ")}
      </Text>
      <Text class="text-lcd-muted" style={{ fontSize: 12 }}>
        {modifierIndicator(props.screen().cursorStyle)}
      </Text>
    </View>
  );
}

/** LCD heights, in points, below which the status bar gives its line to the screen. */
const STATUS_BAR_MIN_HEIGHT = 170;

/** The screen: the status bar (left out when the LCD is short) over the settings screen while `settingsOpen`, else whichever screen the calculator shows. */
export function Lcd(props: { screen: () => Screen; settingsOpen: () => boolean }) {
  const cursorStyle = () => props.screen().cursorStyle;
  const tall = signal(true);
  const calculatorScreen = (
    <View class="flex-1 gap-2">
      <If when={props.screen().home}>{(home) => <HomeView home={home} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().mode}>{(mode) => <ModeView mode={mode} />}</If>
      <If when={props.screen().error}>{(error) => <ErrorView error={error} />}</If>
      <If when={props.screen().functions}>
        {(functions) => <FunctionsView functions={functions} cursorStyle={cursorStyle} />}
      </If>
      <If when={props.screen().fields}>{(fields) => <FieldsView fields={fields} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().table}>{(table) => <TableView table={table} />}</If>
      <If when={props.screen().graph}>{(graph) => <GraphView graph={graph} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().menu}>{(menu) => <MenuView menu={menu} />}</If>
    </View>
  );
  return (
    <View class="bg-bezel rounded-xl p-2" style={{ flexShrink: 1, minHeight: 0 }}>
      <View
        class="bg-lcd rounded-md gap-2 p-3"
        style={{ aspectRatio: 4 / 3, flexShrink: 1, minHeight: 0 }}
        onLayout={(e) => tall.set(e.bounds.size.height >= STATUS_BAR_MIN_HEIGHT)}
      >
        <If when={tall()}>
          <StatusBar screen={props.screen} />
        </If>
        <If when={props.settingsOpen()} else={calculatorScreen}>
          <SettingsView />
        </If>
      </View>
    </View>
  );
}
