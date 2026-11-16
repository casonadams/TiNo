import { signal } from "nsh";
import { Calculator, type Screen } from "tino/native";

const calculator = new Calculator();
const screen = signal<Screen>(calculator.screen);

/** What the calculator's screen shows (tracked where it is read). */
export const currentScreen = (): Screen => screen();

/** Presses the key with id `key`, as the keypad and the keyboard do. */
export function press(key: string): void {
  screen.set(calculator.press(key));
}
