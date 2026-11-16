import { signal } from "nsh";
import { Calculator, type GraphPoint, type Screen } from "tino/native";

const calculator = new Calculator();
const screen = signal<Screen>(calculator.screen);

/** What the calculator's screen shows (tracked where it is read). */
export const currentScreen = (): Screen => screen();

/** Presses the key with id `key`, as the keypad and the keyboard do. */
export function press(key: string): void {
  screen.set(calculator.press(key));
}

/** Drags the graph by fractions of its size (positive `dx` shows more to the right, `dy` more above). */
export function panGraph(dx: number, dy: number): void {
  screen.set(calculator.panGraph(dx, dy));
}

/** Zooms the graph in by `factor` (out below 1) about `anchor`, fractions from the left and bottom. */
export function zoomGraph(factor: number, anchor: GraphPoint): void {
  screen.set(calculator.zoomGraph(factor, anchor));
}

/** Traces at `fraction` of the graph's width from the left. */
export function traceAt(fraction: number): void {
  screen.set(calculator.traceAt(fraction));
}

/** Scrolls the table by `rows` (down when positive). */
export function scrollTable(rows: number): void {
  screen.set(calculator.scrollTable(rows));
}