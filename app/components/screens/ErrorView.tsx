import { Text, View } from "nsh";
import type { ErrorScreen } from "tino/native";

/** The error screen: the error, then its numbered choices with the highlighted one filled. */
export function ErrorView(props: { error: () => ErrorScreen }) {
  return (
    <View class="flex-1 gap-2" testID="error">
      <Text class="text-destructive font-semibold" style={{ fontSize: 20 }}>
        {props.error().message}
      </Text>
      <Each of={props.error().choices}>
        {(choice, index) => (
          <View
            class={`self-start rounded-sm px-1.5 ${index() === props.error().highlighted ? "bg-lcd-foreground" : "bg-lcd"}`}
          >
            <Text
              class={index() === props.error().highlighted ? "text-lcd" : "text-lcd-foreground"}
              style={{ fontSize: 18 }}
            >
              {`${index() + 1}:${choice()}`}
            </Text>
          </View>
        )}
      </Each>
    </View>
  );
}
