import { mount, View } from "nsh";
import { Keypad } from "./components/Keypad";
import { Lcd } from "./components/Lcd";
import { bindPhysicalKeys } from "./input/physicalKeys";
import keypad from "./skins/color/keypad.json";
import type { KeypadLayout } from "./skins/skin";
import { currentScreen, press } from "./store";

function App() {
  bindPhysicalKeys(press);
  return (
    <View
      class="bg-background items-center justify-center p-3"
      style={{
        flex: 1,
        paddingTop: "env(safe-area-inset-top)",
        paddingBottom: "env(safe-area-inset-bottom)",
      }}
    >
      <View class="bg-faceplate w-full flex-1 gap-4 rounded-3xl p-4" style={{ maxWidth: 420, maxHeight: 900 }}>
        <Lcd screen={currentScreen} />
        <Keypad layout={keypad as KeypadLayout} onPress={press} />
      </View>
    </View>
  );
}

mount(App);
