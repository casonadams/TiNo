import { Canvas, type GestureResponderEvent, Text, View } from "nsh";
import type { CalcPanel, CursorStyle, GraphScreen } from "tino/native";
import { panGraph, traceAt, zoomGraph } from "../../store";
import { EntryLine } from "../EntryLine";
import { drawGraph } from "../graphDrawing";
import { GRID, px } from "../lcdGrid";

/** Overlaid graph text: small, on the grid. */
const small = () => ({ fontSize: px(GRID.small) });

/** A CALC item's line: its prompt or result title, the X being typed for value, or its value (`dy/dx=4`). */
function CalcLine(props: { calc: () => CalcPanel; cursorStyle: () => CursorStyle }) {
  return (
    <View class="flex-row items-center" style={{ columnGap: px(6) }} testID="calc">
      <If when={props.calc().heading}>
        {(heading) => (
          <Text class="font-mono text-lcd-foreground" style={small()}>
            {heading()}
          </Text>
        )}
      </If>
      <If when={props.calc().entry}>
        {(entry) => (
          <View class="flex-1 flex-row items-center">
            <Text class="font-mono text-lcd-foreground" style={small()}>
              X=
            </Text>
            <EntryLine
              entry={entry}
              cursor={() => props.calc().entryCursor}
              cursorStyle={props.cursorStyle}
              size={px(GRID.small)}
            />
          </View>
        )}
      </If>
      <If when={props.calc().value}>
        {(value) => (
          <Text class="font-mono text-lcd-foreground" style={small()}>
            {value()}
          </Text>
        )}
      </If>
    </View>
  );
}

/** A drag shorter than this, in points, is a tap: it traces where it lifted. */
const TAP_SLOP = 4;
/** Wheel distance, in points, that zooms by a factor of e. */
const WHEEL_ZOOM_SCALE = 300;
const LINE_HEIGHT = 16;

/** Two fingers on the graph: how far apart they are and the point between them, in points. */
interface Fingers {
  spread: number;
  middle: { x: number; y: number };
}

/** The two fingers down, if two are. */
function fingersOf(e: GestureResponderEvent): Fingers | null {
  const [a, b] = e.touches;
  if (!a || !b) return null;
  return {
    spread: Math.hypot(a.locationX - b.locationX, a.locationY - b.locationY),
    middle: { x: (a.locationX + b.locationX) / 2, y: (a.locationY + b.locationY) / 2 },
  };
}

/** The graph: drag to pan; pinch to zoom about the fingers, which also pan as they move; wheel to zoom about the pointer; tap to trace. The trace's point below it. */
export function GraphView(props: { graph: () => GraphScreen; cursorStyle: () => CursorStyle }) {
  let size = { width: 1, height: 1 };
  let last = { x: 0, y: 0 };
  let travelled = 0;
  let pinched: Fingers | null = null;
  const anchor = (point: { x: number; y: number }) => ({ x: point.x / size.width, y: 1 - point.y / size.height });
  /** One move of two fingers: the graph follows the point between them, then scales about it as they spread. */
  const pinch = (fingers: Fingers) => {
    if (pinched && pinched.spread > 0 && fingers.spread > 0) {
      const dx = fingers.middle.x - pinched.middle.x;
      const dy = fingers.middle.y - pinched.middle.y;
      panGraph(-dx / size.width, dy / size.height);
      zoomGraph(fingers.spread / pinched.spread, anchor(fingers.middle));
    }
    pinched = fingers;
    travelled = Number.POSITIVE_INFINITY;
  };
  return (
    <View class="flex-1">
      <Canvas
        testID="graph"
        style={{ flex: 1 }}
        onLayout={(e) => {
          size = e.bounds.size;
        }}
        draw={(cx, { width, height }) => drawGraph(cx, props.graph(), width, height)}
        onStartShouldSetResponder={() => true}
        onResponderTerminationRequest={() => false}
        onTouchStart={(e) => {
          pinched = fingersOf(e) ?? pinched;
        }}
        onResponderGrant={(e) => {
          last = { x: e.locationX, y: e.locationY };
          travelled = 0;
          pinched = null;
          return true;
        }}
        onResponderMove={(e) => {
          const fingers = fingersOf(e);
          if (fingers) return pinch(fingers);
          if (pinched) {
            pinched = null;
            last = { x: e.locationX, y: e.locationY };
            return;
          }
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
          zoomGraph(Math.exp(-distance / WHEEL_ZOOM_SCALE), anchor(e.position));
        }}
      />
      <If when={props.graph().expression ? props.graph().trace : undefined}>
        {(trace) => (
          <Text
            class="font-mono text-lcd-foreground"
            style={{ ...small(), position: "absolute", top: 0, left: 0 }}
            testID="trace"
          >
            {trace().equation}
          </Text>
        )}
      </If>
      <View style={{ position: "absolute", left: 0, right: 0, bottom: 0 }}>
        <If when={props.graph().calc}>{(calc) => <CalcLine calc={calc} cursorStyle={props.cursorStyle} />}</If>
        <If when={props.graph().trace ?? (props.graph().coordinates ? props.graph().free : undefined)}>
          {(point) => (
            <View class="flex-row justify-between">
              <Text class="font-mono text-lcd-foreground" style={small()}>
                {point().xText}
              </Text>
              <Text class="font-mono text-lcd-foreground" style={small()}>
                {point().yText}
              </Text>
            </View>
          )}
        </If>
      </View>
    </View>
  );
}
