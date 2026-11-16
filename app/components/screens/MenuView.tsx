import { Text, View } from "nsh";
import type { MenuItem, MenuScreen } from "tino/native";

/** A numbered menu: its title, then `digit:label` items with the highlighted one filled. */
export function MenuView(props: { menu: () => MenuScreen }) {
  return (
    <View class="flex-1 gap-0.5" testID="menu">
      <Text class="text-lcd-foreground font-semibold" style={{ fontSize: 15 }}>
        {props.menu().title}
      </Text>
      <Each of={props.menu().items} key={(item: MenuItem) => item.digit}>
        {(item, index) => (
          <View
            class={`self-start rounded-sm px-1.5 ${index() === props.menu().highlighted ? "bg-lcd-foreground" : "bg-lcd"}`}
          >
            <Text
              class={index() === props.menu().highlighted ? "text-lcd" : "text-lcd-foreground"}
              style={{ fontSize: 17 }}
            >
              {`${item().digit}:${item().label}`}
            </Text>
          </View>
        )}
      </Each>
    </View>
  );
}
