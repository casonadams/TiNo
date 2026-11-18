import { signal, Storage } from "nsh";
import { Calculator, type GraphPoint, type Screen } from "tino/native";

/** The storage key the calculator is kept under between sessions. */
const CALCULATOR_KEY = "tino.calculator";

const calculator = new Calculator();
const screen = signal<Screen>(calculator.screen);

/** What the calculator's screen shows (tracked where it is read). */
export const currentScreen = (): Screen => screen();

/** Whether a program's next slice is already scheduled. */
let resuming = false;

/** Runs the busy program's next slice. */
function resume(): void {
  resuming = false;
  show(calculator.resume());
}

/** Shows `next` and keeps the calculator for the next session; while a program is busy, carries it on after this update instead. */
function show(next: Screen): void {
  screen.set(next);
  if (!next.busy) {
    void Storage.setItem(CALCULATOR_KEY, calculator.save());
  } else if (!resuming) {
    resuming = true;
    setTimeout(resume, 0);
  }
}

/** Brings back the calculator the last session kept; a kept one this version cannot read is replaced. */
export async function restoreCalculator(): Promise<void> {
  const kept = await Storage.getItem(CALCULATOR_KEY);
  if (kept === null) return;
  try {
    screen.set(calculator.restore(kept));
  } catch (error) {
    console.warn(`starting a new calculator: ${error}`);
  }
}

/** Presses the key with id `key`, as the keypad and the keyboard do. */
export function press(key: string): void {
  show(calculator.press(key));
}

/** Drags the graph by fractions of its size (positive `dx` shows more to the right, `dy` more above). */
export function panGraph(dx: number, dy: number): void {
  show(calculator.panGraph(dx, dy));
}

/** Zooms the graph in by `factor` (out below 1) about `anchor`, fractions from the left and bottom. */
export function zoomGraph(factor: number, anchor: GraphPoint): void {
  show(calculator.zoomGraph(factor, anchor));
}

/** Traces at `fraction` of the graph's width from the left. */
export function traceAt(fraction: number): void {
  screen.set(calculator.traceAt(fraction));
}

/** Scrolls the table by `rows` (down when positive). */
export function scrollTable(rows: number): void {
  screen.set(calculator.scrollTable(rows));
}
