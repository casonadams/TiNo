import { Text, View } from "nsh";
import type { ErrorScreen } from "tino/native";
import { GRID, px } from "../lcdGrid";

/** The error screen: the error, then its numbered choices, the highlighted choice's number filled. */
export function ErrorView(props: { error: () => ErrorScreen }) {
  const text = () => ({ fontSize: px(GRID.text) });
  const line = () => ({ height: px(GRID.line) });
  return (
    <View class="flex-1" testID="error">
      <View class="justify-center" style={line()}>
        <Text class="text-lcd-error font-mono" style={text()}>
          {props.error().message}
        </Text>
      </View>
      <Each of={props.error().choices}>
        {(choice, index) => (
          <View class="flex-row items-center" style={line()} accessibilityLabel={`${index() + 1}:${choice()}`}>
            <View class={index() === props.error().highlighted ? "bg-lcd-foreground" : ""}>
              <Text
                class={`font-mono ${index() === props.error().highlighted ? "text-lcd" : "text-lcd-foreground"}`}
                style={text()}
              >
                {`${index() + 1}:`}
              </Text>
            </View>
            <Text class="font-mono text-lcd-foreground" style={text()}>
              {choice()}
            </Text>
          </View>
        )}
      </Each>
    </View>
  );
}
