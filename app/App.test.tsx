import { expect, open, test } from "nsh/test";
import { Calculator } from "tino/native";
import keypad from "./skins/color/keypad.json";
import { keysOf, type KeypadLayout } from "./skins/skin";

test("keypad presses type tokens on the entry line", async () => {
  const app = await open("./App.tsx");
  await app.press("id:key-7");
  await app.press("id:key-add");
  await app.press("id:key-second");
  expect(app.texts()).toContain("2ND");
  await app.press("id:key-sin");
  expect(app.shows("label:7+sin⁻¹(")).toBe(true);
  expect(app.texts()).not.toContain("2ND");
});

test("enter shows the result and clears the entry line", async () => {
  const app = await open("./App.tsx");
  for (const key of ["1", "2", "multiply", "1", "2", "enter"]) await app.press(`id:key-${key}`);
  expect(app.texts()).toContain("144");
  expect(app.shows("label:12×12")).toBe(false);
});

test("an error is shown until the next key press", async () => {
  const app = await open("./App.tsx");
  for (const key of ["1", "divide", "0", "enter"]) await app.press(`id:key-${key}`);
  expect(app.texts()).toContain("ERR:DIVIDE BY 0");
  await app.press("id:key-del");
  expect(app.texts()).not.toContain("ERR:DIVIDE BY 0");
});

test("the computer keyboard presses calculator keys", async () => {
  const app = await open("./App.tsx");
  await app.keyboard.press("4");
  await app.keyboard.press("shift+6");
  await app.keyboard.press("2");
  await app.keyboard.press("backspace");
  expect(app.shows("label:4^")).toBe(true);
  await app.keyboard.press("escape");
  expect(app.shows("label:4^")).toBe(false);
});

test("every key of the skin is a key the calculator knows", () => {
  const calculator = new Calculator();
  for (const spec of keysOf(keypad as KeypadLayout)) {
    expect(() => calculator.press(spec.key)).not.toThrow();
  }
});
