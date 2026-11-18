import { Text, View } from "nsh";
import type { Screen } from "tino/native";
import { modifierIndicator } from "./lcdCells";
import { fitGrid, GRID, px } from "./lcdGrid";
import { ErrorView } from "./screens/ErrorView";
import { FieldsView } from "./screens/FieldsView";
import { FunctionsView } from "./screens/FunctionsView";
import { GraphView } from "./screens/GraphView";
import { HomeView } from "./screens/HomeView";
import { MenuView } from "./screens/MenuView";
import { ModeView } from "./screens/ModeView";
import { SettingsView } from "./screens/SettingsView";
import { TableView } from "./screens/TableView";

/** Whether ▲ ▼ highlight a history entry or result: the status bar then reads HISTORY, as the mode may have differed. */
function inHistory(screen: Screen): boolean {
  return screen.home?.history.some((line) => line.entrySelected || line.resultSelected) ?? false;
}

/** The status band: the mode settings (HISTORY while in the history), then the pending modifier. */
function StatusBar(props: { screen: () => Screen }) {
  const text = { fontSize: px(GRID.statusText) };
  return (
    <View
      class="bg-lcd-status flex-row items-center justify-between"
      style={{ height: px(GRID.status), paddingHorizontal: px(GRID.inset) }}
    >
      <Text class="text-lcd-muted font-mono" style={text}>
        {inHistory(props.screen()) ? "HISTORY" : props.screen().status.join(" ")}
      </Text>
      <Text class="text-lcd-muted font-mono" style={text}>
        {modifierIndicator(props.screen().cursorStyle)}
      </Text>
    </View>
  );
}

/** The screen, on the calculator's 320×240 grid: the status band over the settings screen while `settingsOpen`, else whichever screen the calculator shows. */
export function Lcd(props: { screen: () => Screen; settingsOpen: () => boolean }) {
  const cursorStyle = () => props.screen().cursorStyle;
  const calculatorScreen = (
    <View class="flex-1">
      <If when={props.screen().home}>{(home) => <HomeView home={home} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().mode}>{(mode) => <ModeView mode={mode} />}</If>
      <If when={props.screen().error}>{(error) => <ErrorView error={error} />}</If>
      <If when={props.screen().functions}>
        {(functions) => <FunctionsView functions={functions} cursorStyle={cursorStyle} />}
      </If>
      <If when={props.screen().fields}>{(fields) => <FieldsView fields={fields} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().table}>{(table) => <TableView table={table} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().graph}>{(graph) => <GraphView graph={graph} cursorStyle={cursorStyle} />}</If>
      <If when={props.screen().menu}>{(menu) => <MenuView menu={menu} />}</If>
    </View>
  );
  return (
    <View class="bg-bezel rounded-xl p-2" style={{ flexShrink: 1, minHeight: 0 }}>
      <View
        class="bg-lcd rounded-md"
        style={{ aspectRatio: 4 / 3, flexShrink: 1, minHeight: 0, overflow: "hidden" }}
        onLayout={(e) => fitGrid(e.bounds.size.width)}
      >
        <StatusBar screen={props.screen} />
        <View class="flex-1" style={{ paddingHorizontal: px(GRID.inset), paddingVertical: px(2) }}>
          <If when={props.settingsOpen()} else={calculatorScreen}>
            <SettingsView />
          </If>
        </View>
      </View>
    </View>
  );
}
