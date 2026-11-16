import { Text, View } from "nsh";
import type { KeySpec, KeyVariant } from "../skins/skin";

const SURFACES: Record<KeyVariant, string> = {
  default: "bg-key",
  function: "bg-key-function",
  number: "bg-key-number",
  operator: "bg-key-operator",
  second: "bg-key-second",
  alpha: "bg-key-alpha",
};

const LABELS: Record<KeyVariant, string> = {
  default: "text-key-foreground",
  function: "text-key-function-foreground",
  number: "text-key-number-foreground",
  operator: "text-key-operator-foreground",
  second: "text-key-second-foreground",
  alpha: "text-key-alpha-foreground",
};

/** A key with its 2nd legend above on the left and its ALPHA legend above on the right. */
export function KeyButton(props: { spec: KeySpec; onPress: (key: string) => void; legends?: boolean }) {
  const variant = props.spec.variant ?? "default";
  return (
    <View class="flex-1 gap-0.5">
      <If when={props.legends !== false}>
        <View class="flex-row justify-between px-0.5" style={{ height: 12 }}>
          <Text class="text-legend-second" style={{ fontSize: 9 }}>
            {props.spec.second ?? ""}
          </Text>
          <Text class="text-legend-alpha" style={{ fontSize: 9 }}>
            {props.spec.alpha ?? ""}
          </Text>
        </View>
      </If>
      <View
        role="button"
        accessibilityLabel={props.spec.label}
        testID={`key-${props.spec.key}`}
        onPress={() => props.onPress(props.spec.key)}
        class={`flex-1 items-center justify-center rounded-md active:opacity-70 ${SURFACES[variant]}`}
        style={{ minHeight: 28 }}
      >
        <Text class={`font-semibold ${LABELS[variant]}`} style={{ fontSize: 15 }}>
          {props.spec.label}
        </Text>
      </View>
    </View>
  );
}
