import { View } from "nsh";
import type { KeypadLayout, KeySpec } from "../skins/skin";
import { KeyButton } from "./KeyButton";

type Press = (key: string) => void;

function KeyRow(props: { keys: KeySpec[]; onPress: Press }) {
  return (
    <View class="flex-1 flex-row gap-2">
      {props.keys.map((spec) => (
        <KeyButton spec={spec} onPress={props.onPress} />
      ))}
    </View>
  );
}

function Dpad(props: { dpad: KeypadLayout["dpad"]; onPress: Press }) {
  const arrow = (spec: KeySpec) => <KeyButton spec={spec} onPress={props.onPress} legends={false} />;
  return (
    <View class="gap-1" style={{ flex: 2 }}>
      <View class="flex-1 flex-row gap-1">
        <View class="flex-1" />
        {arrow(props.dpad.up)}
        <View class="flex-1" />
      </View>
      <View class="flex-1 flex-row gap-1">
        {arrow(props.dpad.left)}
        <View class="flex-1" />
        {arrow(props.dpad.right)}
      </View>
      <View class="flex-1 flex-row gap-1">
        <View class="flex-1" />
        {arrow(props.dpad.down)}
        <View class="flex-1" />
      </View>
    </View>
  );
}

/** The keypad a skin lays out: the function row, 2nd/ALPHA beside the arrows, then the main grid. */
export function Keypad(props: { layout: KeypadLayout; onPress: Press }) {
  return (
    <View class="gap-1.5" style={{ flexGrow: 1, flexShrink: 1, minHeight: 0 }}>
      <KeyRow keys={props.layout.functionRow} onPress={props.onPress} />
      <View class="flex-row gap-2" style={{ flex: 2 }}>
        <View class="gap-1.5" style={{ flex: 3 }}>
          {props.layout.control.map((row) => (
            <KeyRow keys={row} onPress={props.onPress} />
          ))}
        </View>
        <Dpad dpad={props.layout.dpad} onPress={props.onPress} />
      </View>
      {props.layout.main.map((row) => (
        <KeyRow keys={row} onPress={props.onPress} />
      ))}
    </View>
  );
}
