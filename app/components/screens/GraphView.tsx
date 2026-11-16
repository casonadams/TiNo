import { Canvas, createAnimatedValue, Text, View } from "nsh";
import type { GraphScreen } from "tino/native";
import { panGraph, traceAt, zoomGraph } from "../../store";
import { drawGraph } from "../graphDrawing";

/** A drag shorter than this, in points, is a tap: it traces where it lifted. */
const TAP_SLOP = 4;
/** Wheel distance, in points, that zooms by a factor of e. */
const WHEEL_ZOOM_SCALE = 300;
const LINE_HEIGHT = 16;

/** The graph: drag to pan, wheel or pinch to zoom, tap to trace; the trace's point below it. */
export function GraphView(props: { graph: () => GraphScreen }) {
  let size = { width: 1, height: 1 };
  let last = { x: 0, y: 0 };
  let travelled = 0;
  const pinch = createAnimatedValue(1);
  return (
    <View class="flex-1 gap-1">
      <Canvas
        testID="graph"
        style={{ flex: 1 }}
        onLayout={(e) => {
          size = e.bounds.size;
        }}
        draw={(cx, { width, height }) => drawGraph(cx, props.graph(), width, height)}
        onStartShouldSetResponder={() => true}
        onResponderTerminationRequest={() => false}
        onResponderGrant={(e) => {
          last = { x: e.locationX, y: e.locationY };
          travelled = 0;
          return true;
        }}
        onResponderMove={(e) => {
          const dx = e.locationX - last.x;
          const dy = e.locationY - last.y;
          travelled += Math.hypot(dx, dy);
          last = { x: e.locationX, y: e.locationY };
          if (travelled > TAP_SLOP) panGraph(-dx / size.width, dy / size.height);
        }}
        onResponderRelease={(e) => {
          if (travelled <= TAP_SLOP) traceAt(e.locationX / size.width);
        }}
        onWheel={(e) => {
          const distance = e.delta.y * (e.deltaMode === "lines" ? LINE_HEIGHT : 1);
          const anchor = { x: e.position.x / size.width, y: 1 - e.position.y / size.height };
          zoomGraph(Math.exp(-distance / WHEEL_ZOOM_SCALE), anchor);
        }}
        pinch={{ scale: pinch }}
        onPinchEnd={(e) => {
          zoomGraph(e.scale, { x: 0.5, y: 0.5 });
          pinch.set(1);
        }}
      />
      <If when={props.graph().trace}>
        {(trace) => (
          <View class="gap-0.5" testID="trace">
            <Text class="text-lcd-foreground" style={{ fontSize: 13 }}>
              {trace().equation}
            </Text>
            <View class="flex-row justify-between">
              <Text class="text-lcd-foreground" style={{ fontSize: 13 }}>
                {trace().xText}
              </Text>
              <Text class="text-lcd-foreground" style={{ fontSize: 13 }}>
                {trace().yText}
              </Text>
            </View>
          </View>
        )}
      </If>
    </View>
  );
}
