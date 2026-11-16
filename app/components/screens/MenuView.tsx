import { Text, View } from "nsh";
import type { MenuItem, MenuScreen } from "tino/native";

/** A menu: its tab titles with the shown one filled, then `key:label` items with the highlighted one filled. */
export function MenuView(props: { menu: () => MenuScreen }) {
  return (
    <View class="flex-1 gap-0.5" testID="menu">
      <View class="flex-row gap-2">
        <Each of={props.menu().tabs} key={(tab: string) => tab}>
          {(tab, index) => (
            <View class={`rounded-sm px-1 ${index() === props.menu().tab ? "bg-lcd-foreground" : "bg-lcd"}`}>
              <Text
                class={`font-semibold ${index() === props.menu().tab ? "text-lcd" : "text-lcd-foreground"}`}
                style={{ fontSize: 15 }}
              >
                {tab()}
              </Text>
            </View>
          )}
        </Each>
      </View>
      <Each of={props.menu().items} key={(item: MenuItem) => item.key}>
        {(item, index) => (
          <View
            class={`self-start rounded-sm px-1.5 ${index() === props.menu().highlighted ? "bg-lcd-foreground" : "bg-lcd"}`}
          >
            <Text
              class={index() === props.menu().highlighted ? "text-lcd" : "text-lcd-foreground"}
              style={{ fontSize: 17 }}
            >
              {`${item().key}:${item().label}`}
            </Text>
          </View>
        )}
      </Each>
    </View>
  );
}
