import { Text, View } from "nsh";
import { SKINS } from "../../skins";
import { chooseSkin, currentSkin } from "../../skinStore";

/** The settings screen ON opens: the skins, the shown one filled; tap one or use ▲ ▼, then ON, CLEAR or ENTER to return. */
export function SettingsView() {
  return (
    <View class="flex-1 gap-2" testID="settings">
      <Text class="text-lcd-foreground font-semibold" style={{ fontSize: 15 }}>
        SETTINGS
      </Text>
      <Text class="text-lcd-muted" style={{ fontSize: 13 }}>
        Skin
      </Text>
      {SKINS.map((skin) => (
        <View
          role="button"
          accessibilityLabel={`${skin.name} skin`}
          accessibilityState={{ selected: currentSkin().id === skin.id }}
          testID={`skin-${skin.id}`}
          onPress={() => chooseSkin(skin.id)}
          class={`self-start rounded-sm px-1.5 active:opacity-70 ${
            currentSkin().id === skin.id ? "bg-lcd-foreground" : "bg-lcd"
          }`}
        >
          <Text
            class={currentSkin().id === skin.id ? "text-lcd" : "text-lcd-foreground"}
            style={{ fontSize: 17 }}
          >
            {skin.name}
          </Text>
        </View>
      ))}
      <Text class="text-lcd-muted" style={{ fontSize: 12 }}>
        ON, CLEAR or ENTER: back
      </Text>
    </View>
  );
}
