import { Text, View } from "nsh";
import type { Screen } from "tino/native";
import { modifierIndicator } from "./lcdCells";
import { ErrorView } from "./screens/ErrorView";
import { HomeView } from "./screens/HomeView";
import { ModeView } from "./screens/ModeView";

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

/** The screen: the status bar over whichever screen the calculator shows. */
export function Lcd(props: { screen: () => Screen }) {
  return (
    <View class="bg-bezel rounded-xl p-3">
      <View class="bg-lcd rounded-md gap-2 p-3" style={{ aspectRatio: 4 / 3 }}>
        <StatusBar screen={props.screen} />
        <If when={props.screen().home}>
          {(home) => <HomeView home={home} cursorStyle={() => props.screen().cursorStyle} />}
        </If>
        <If when={props.screen().mode}>{(mode) => <ModeView mode={mode} />}</If>
        <If when={props.screen().error}>{(error) => <ErrorView error={error} />}</If>
      </View>
    </View>
  );
}
