import { type CanvasRenderingContext2D, token } from "nsh";
import type { GraphScreen } from "tino/native";

/** Colors of the curves by Y= slot: `Y1` first, `Y0` last. */
const CURVE_TOKENS = [
  "graph-1",
  "graph-2",
  "graph-3",
  "graph-4",
  "graph-5",
  "graph-6",
  "graph-7",
  "graph-8",
  "graph-9",
  "graph-10",
] as const;

const TICK = 4;

/** Graph coordinates to the canvas's points. */
interface Scale {
  x: (x: number) => number;
  y: (y: number) => number;
}

/** Maps graph coordinates to the canvas's points. */
function scales(graph: GraphScreen, width: number, height: number): Scale {
  return {
    x: (x: number) => ((x - graph.xMin) / (graph.xMax - graph.xMin)) * width,
    y: (y: number) => height - ((y - graph.yMin) / (graph.yMax - graph.yMin)) * height,
  };
}

/** ∫f(x)dx's area: between the traced curve and y=0 over the shaded interval, in the curve's color. */
function drawShade(cx: CanvasRenderingContext2D, graph: GraphScreen, scale: Scale) {
  const { calc, trace } = graph;
  if (!calc || !trace || calc.shadeFrom == null || calc.shadeTo == null) return;
  const curve = graph.curves.find((candidate) => candidate.slot === trace.slot);
  if (!curve) return;
  const [from, to] = [Math.min(calc.shadeFrom, calc.shadeTo), Math.max(calc.shadeFrom, calc.shadeTo)];
  const baseline = scale.y(0);
  cx.fillStyle = token(CURVE_TOKENS[curve.slot % CURVE_TOKENS.length]);
  cx.globalAlpha = 0.3;
  cx.beginPath();
  cx.moveTo(scale.x(from), baseline);
  graph.xs.forEach((x, i) => {
    const y = curve.ys[i];
    if (x >= from && x <= to && y != null) cx.lineTo(scale.x(x), scale.y(y));
  });
  cx.lineTo(scale.x(to), baseline);
  cx.closePath();
  cx.fill();
  cx.globalAlpha = 1;
}

/** The canvas's size in points. */
interface CanvasSize {
  width: number;
  height: number;
}

/** FORMAT's grid: a dot at each tick crossing, or lines through the ticks. */
function drawGrid(cx: CanvasRenderingContext2D, graph: GraphScreen, scale: Scale, size: CanvasSize) {
  if (graph.grid === "off") return;
  cx.strokeStyle = token("graph-grid");
  cx.fillStyle = token("graph-grid");
  cx.lineWidth = 1;
  if (graph.grid === "dot") {
    for (const x of graph.xTicks) for (const y of graph.yTicks) cx.fillRect(scale.x(x) - 1, scale.y(y) - 1, 2, 2);
    return;
  }
  cx.beginPath();
  for (const x of graph.xTicks) {
    cx.moveTo(scale.x(x), 0);
    cx.lineTo(scale.x(x), size.height);
  }
  for (const y of graph.yTicks) {
    cx.moveTo(0, scale.y(y));
    cx.lineTo(size.width, scale.y(y));
  }
  cx.stroke();
}

/** The axes with their ticks, and with LabelOn an x and a y at their ends. */
function drawAxes(cx: CanvasRenderingContext2D, graph: GraphScreen, scale: Scale, size: CanvasSize) {
  if (!graph.axes) return;
  cx.strokeStyle = token("graph-axis");
  cx.fillStyle = token("graph-axis");
  cx.lineWidth = 1;
  cx.font = "12px monospace";
  cx.beginPath();
  if (graph.yMin <= 0 && graph.yMax >= 0) {
    const axis = scale.y(0);
    cx.moveTo(0, axis);
    cx.lineTo(size.width, axis);
    for (const tick of graph.xTicks) {
      cx.moveTo(scale.x(tick), axis - TICK);
      cx.lineTo(scale.x(tick), axis + TICK);
    }
    if (graph.labels) cx.fillText("x", size.width - 10, axis - 6);
  }
  if (graph.xMin <= 0 && graph.xMax >= 0) {
    const axis = scale.x(0);
    cx.moveTo(axis, 0);
    cx.lineTo(axis, size.height);
    for (const tick of graph.yTicks) {
      cx.moveTo(axis - TICK, scale.y(tick));
      cx.lineTo(axis + TICK, scale.y(tick));
    }
    if (graph.labels) cx.fillText("y", axis + 6, 12);
  }
  cx.stroke();
}

function drawCurves(cx: CanvasRenderingContext2D, graph: GraphScreen, scale: Scale) {
  cx.lineWidth = 2;
  cx.lineJoin = "round";
  for (const curve of graph.curves) {
    cx.strokeStyle = token(CURVE_TOKENS[curve.slot % CURVE_TOKENS.length]);
    cx.beginPath();
    let drawing = false;
    curve.ys.forEach((y, i) => {
      if (y === null || y === undefined) {
        drawing = false;
        return;
      }
      const at: [number, number] = [scale.x(graph.xs[i]), scale.y(y)];
      if (drawing) cx.lineTo(...at);
      else cx.moveTo(...at);
      drawing = true;
    });
    cx.stroke();
  }
}

/** The trace cursor as a ring on its curve; the free cursor as a cross, with ZBox's box from the marked corner. */
function drawCursors(cx: CanvasRenderingContext2D, graph: GraphScreen, scale: Scale) {
  cx.strokeStyle = token("graph-trace");
  cx.lineWidth = 1.5;
  const { trace, free } = graph;
  if (trace && trace.y != null) {
    cx.beginPath();
    cx.arc(scale.x(trace.x), scale.y(trace.y), 5, 0, Math.PI * 2);
    cx.stroke();
  }
  if (!free) return;
  const [x, y] = [scale.x(free.x), scale.y(free.y)];
  cx.beginPath();
  cx.moveTo(x - 6, y);
  cx.lineTo(x + 6, y);
  cx.moveTo(x, y - 6);
  cx.lineTo(x, y + 6);
  if (free.cornerX != null && free.cornerY != null) {
    const [cornerX, cornerY] = [scale.x(free.cornerX), scale.y(free.cornerY)];
    cx.rect(Math.min(x, cornerX), Math.min(y, cornerY), Math.abs(x - cornerX), Math.abs(y - cornerY));
  }
  cx.stroke();
}

/** Draws the graph as FORMAT says: ∫f(x)dx's shading, the grid, the axes, each curve in its slot's color, and the cursors. */
export function drawGraph(cx: CanvasRenderingContext2D, graph: GraphScreen, width: number, height: number) {
  const scale = scales(graph, width, height);
  const size = { width, height };
  drawShade(cx, graph, scale);
  drawGrid(cx, graph, scale, size);
  drawAxes(cx, graph, scale, size);
  drawCurves(cx, graph, scale);
  drawCursors(cx, graph, scale);
}
