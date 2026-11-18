import { Text, View } from "nsh";
import { hapticsOn, setHaptics } from "../../haptics";
import { type SettingsLine, settingsLine } from "../../settings";
import { SKINS } from "../../skins";
import { chooseSkin, currentSkin } from "../../skinStore";
import { GRID, px } from "../lcdGrid";

/** One option of a settings line: filled when chosen, outlined on the cursor's line; tapping chooses it. */
function Choice(props: { label: string; chosen: boolean; line: SettingsLine; testID: string; onPress: () => void }) {
  return (
    <View
      role="button"
      accessibilityLabel={props.label}
      accessibilityState={{ selected: props.chosen }}
      testID={props.testID}
      onPress={props.onPress}
      class={`border active:opacity-70 ${settingsLine() === props.line && props.chosen ? "border-lcd-accent" : "border-transparent"} ${
        props.chosen ? "bg-lcd-foreground" : ""
      }`}
      style={{ paddingHorizontal: px(2) }}
    >
      <Text
        class={`font-mono ${props.chosen ? "text-lcd" : "text-lcd-foreground"}`}
        style={{ fontSize: px(GRID.text) }}
      >
        {props.label}
      </Text>
    </View>
  );
}

/** The settings screen ON opens, a line per setting as MODE draws them: the skin, and key haptics. Tap an option or use the arrows, then ON, CLEAR or ENTER to return. */
export function SettingsView() {
  const text = () => ({ fontSize: px(GRID.text) });
  const line = () => ({ minHeight: px(GRID.line), columnGap: px(4) });
  return (
    <View class="flex-1" testID="settings">
      <View class="flex-row flex-wrap items-center" style={line()}>
        <Text class="font-mono text-lcd-muted" style={text()}>
          Skin:
        </Text>
        {SKINS.map((skin) => (
          <Choice
            label={skin.name}
            chosen={currentSkin().id === skin.id}
            line="skin"
            testID={`skin-${skin.id}`}
            onPress={() => chooseSkin(skin.id)}
          />
        ))}
      </View>
      <View class="flex-row flex-wrap items-center" style={line()}>
        <Text class="font-mono text-lcd-muted" style={text()}>
          Haptics:
        </Text>
        <Choice label="On" chosen={hapticsOn()} line="haptics" testID="haptics-on" onPress={() => setHaptics(true)} />
        <Choice label="Off" chosen={!hapticsOn()} line="haptics" testID="haptics-off" onPress={() => setHaptics(false)} />
      </View>
      <View class="justify-center" style={{ height: px(GRID.line) }}>
        <Text class="font-mono text-lcd-muted" style={{ fontSize: px(GRID.small) }}>
          ON, CLEAR or ENTER: back
        </Text>
      </View>
    </View>
  );
}
