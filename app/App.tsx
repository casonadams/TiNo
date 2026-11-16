import { mount, View } from "nsh";
import { Keypad } from "./components/Keypad";
import { Lcd } from "./components/Lcd";
import { SkinPicker } from "./components/SkinPicker";
import { bindPhysicalKeys } from "./input/physicalKeys";
import { currentSkin, restoreSkin } from "./skinStore";
import { currentScreen, press, restoreCalculator } from "./store";

function App() {
  bindPhysicalKeys(press);
  void restoreCalculator();
  void restoreSkin();
  return (
    <View
      class="bg-background items-center justify-center gap-2 p-3"
      style={{
        flex: 1,
        paddingTop: "env(safe-area-inset-top)",
        paddingBottom: "env(safe-area-inset-bottom)",
      }}
    >
      <SkinPicker />
      <View
        class="bg-faceplate border-faceplate-foreground/15 w-full flex-1 gap-4 rounded-3xl border p-4 shadow-2xl"
        style={{ maxWidth: 420, maxHeight: 900 }}
      >
        <Lcd screen={currentScreen} />
        <Keypad layout={currentSkin().keypad} onPress={press} />
      </View>
    </View>
  );
}

mount(App);
