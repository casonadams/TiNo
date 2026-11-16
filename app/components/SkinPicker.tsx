import { Text, View } from "nsh";
import { SKINS } from "../skins";
import { chooseSkin, currentSkin } from "../skinStore";

/** A row of the skins by name; pressing one shows it. */
export function SkinPicker() {
  return (
    <View class="flex-row items-center justify-center gap-2" style={{ height: 28 }}>
      {SKINS.map((skin) => (
        <View
          role="button"
          accessibilityLabel={`${skin.name} skin`}
          accessibilityState={{ selected: currentSkin().id === skin.id }}
          testID={`skin-${skin.id}`}
          onPress={() => chooseSkin(skin.id)}
          class={`rounded-full px-3 py-1 active:opacity-70 ${
            currentSkin().id === skin.id ? "bg-faceplate" : "bg-background border border-faceplate"
          }`}
        >
          <Text
            class={currentSkin().id === skin.id ? "text-faceplate-foreground" : "text-foreground"}
            style={{ fontSize: 13 }}
          >
            {skin.name}
          </Text>
        </View>
      ))}
    </View>
  );
}
