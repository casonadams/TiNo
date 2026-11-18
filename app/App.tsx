import { mount, View } from "nsh";
import { Keypad } from "./components/Keypad";
import { Lcd } from "./components/Lcd";
import { keyTap, restoreHaptics } from "./haptics";
import { bindPhysicalKeys } from "./input/physicalKeys";
import { pressKey, settingsOpen } from "./settings";
import { currentSkin, restoreSkin } from "./skinStore";
import { currentScreen, restoreCalculator } from "./store";

/** A key of the on-screen keypad: a haptic under the finger, then the press. */
function tapKey(key: string): void {
  keyTap();
  pressKey(key);
}

function App() {
  bindPhysicalKeys(pressKey);
  void restoreCalculator();
  void restoreSkin();
  void restoreHaptics();
  return (
    <View
      class="bg-background items-center justify-center gap-2 p-3"
      style={{
        flex: 1,
        paddingTop: "env(safe-area-inset-top)",
        paddingBottom: "env(safe-area-inset-bottom)",
      }}
    >
      <View
        class="bg-faceplate border-faceplate-foreground/15 w-full flex-1 gap-3 rounded-3xl border p-3 shadow-2xl"
        style={{ maxWidth: 420, maxHeight: 900 }}
      >
        <Lcd screen={currentScreen} settingsOpen={settingsOpen} />
        <Keypad layout={currentSkin().keypad} onPress={tapKey} />
      </View>
    </View>
  );
}

mount(App);
