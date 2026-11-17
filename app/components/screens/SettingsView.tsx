import { Text, View } from "nsh";
import { SKINS } from "../../skins";
import { chooseSkin, currentSkin } from "../../skinStore";
import { GRID, px } from "../lcdGrid";

/** The settings screen ON opens: the skins, the shown one filled; tap one or use ▲ ▼, then ON, CLEAR or ENTER to return. */
export function SettingsView() {
  const text = () => ({ fontSize: px(GRID.text) });
  const line = () => ({ height: px(GRID.line) });
  return (
    <View class="flex-1" testID="settings">
      <View class="justify-center" style={line()}>
        <Text class="font-mono text-lcd-foreground" style={text()}>
          SETTINGS
        </Text>
      </View>
      <View class="justify-center" style={line()}>
        <Text class="font-mono text-lcd-muted" style={text()}>
          Skin:
        </Text>
      </View>
      {SKINS.map((skin) => (
        <View
          role="button"
          accessibilityLabel={`${skin.name} skin`}
          accessibilityState={{ selected: currentSkin().id === skin.id }}
          testID={`skin-${skin.id}`}
          onPress={() => chooseSkin(skin.id)}
          class={`self-start justify-center active:opacity-70 ${currentSkin().id === skin.id ? "bg-lcd-foreground" : ""}`}
          style={{ ...line(), paddingHorizontal: px(2) }}
        >
          <Text
            class={`font-mono ${currentSkin().id === skin.id ? "text-lcd" : "text-lcd-foreground"}`}
            style={text()}
          >
            {skin.name}
          </Text>
        </View>
      ))}
      <View class="justify-center" style={line()}>
        <Text class="font-mono text-lcd-muted" style={{ fontSize: px(GRID.small) }}>
          ON, CLEAR or ENTER: back
        </Text>
      </View>
    </View>
  );
}
