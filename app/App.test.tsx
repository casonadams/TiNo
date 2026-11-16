import { expect, open, test } from "nsh/test";
import { SKINS } from "./skins";
import { keysOf } from "./skins/skin";

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

test("MATH shows its tabs and an item's key types it into the entry", async () => {
  const app = await open("./App.tsx");
  await app.press("id:key-math");
  for (const text of ["MATH", "NUM", "PROB", "1:►Frac", "A:logBASE("]) expect(app.texts()).toContain(text);
  await app.press("id:key-right");
  expect(app.texts()).toContain("0:remainder(");
  await app.press("id:key-1");
  expect(app.shows("id:menu")).toBe(false);
  expect(app.shows("label:abs(")).toBe(true);
});

test("an error offers Quit and Goto; Goto returns to the entry", async () => {
  const app = await open("./App.tsx");
  for (const key of ["1", "divide", "0", "enter"]) await app.press(`id:key-${key}`);
  for (const text of ["ERR:DIVIDE BY 0", "1:Quit", "2:Goto"]) expect(app.texts()).toContain(text);
  await app.press("id:key-2");
  expect(app.texts()).not.toContain("ERR:DIVIDE BY 0");
  expect(app.shows("label:1÷0")).toBe(true);
});

test("MODE changes how results are written and the status bar says so", async () => {
  const app = await open("./App.tsx");
  for (const key of ["mode", "down", "right", "right", "right", "enter", "second", "mode"]) {
    await app.press(`id:key-${key}`);
  }
  for (const key of ["2", "divide", "3", "enter"]) await app.press(`id:key-${key}`);
  for (const text of [".67", "NORMAL FIX2 RADIAN"]) expect(app.texts()).toContain(text);
});

test("a function typed in Y= graphs, and TRACE reports its points", async () => {
  const app = await open("./App.tsx");
  for (const key of ["y-equals", "xt-theta-n", "square", "trace"]) await app.press(`id:key-${key}`);
  expect(app.shows("id:graph")).toBe(true);
  for (const text of ["Y1=X²", "X=0", "Y=0"]) expect(app.texts()).toContain(text);
  await app.press("id:key-right");
  expect(app.texts()).not.toContain("X=0");
});

test("dragging the graph moves the window; tapping it traces", async () => {
  const app = await open("./App.tsx");
  for (const key of ["y-equals", "xt-theta-n", "graph"]) await app.press(`id:key-${key}`);
  await app.drag("id:graph", { x: 0, y: 0 });
  expect(app.texts()).toContain("Y1=X");
  await app.drag("id:graph", { x: -60, y: 0 });
  await app.press("id:key-window");
  const texts = app.texts();
  expect(texts[texts.indexOf("Xmax=") + 1]).not.toBe("10");
});

test("TABLE lists each graphed function by X; arrows select cells and dragging scrolls", async () => {
  const app = await open("./App.tsx");
  for (const key of ["y-equals", "xt-theta-n", "square", "second", "graph"]) await app.press(`id:key-${key}`);
  for (const text of ["X", "Y1", "X=0"]) expect(app.texts()).toContain(text);
  await app.press("id:key-down");
  await app.press("id:key-right");
  expect(app.texts()).toContain("Y1=1");
  await app.drag("id:table", { x: 0, y: -60 });
  expect(app.texts()).toContain("Y1=16");
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

test("the calculator is kept between sessions", async () => {
  const first = await open("./App.tsx");
  for (const key of ["1", "2", "3", "enter", "mode", "down", "down", "right", "enter", "second", "mode"]) {
    await first.press(`id:key-${key}`);
  }
  const second = await open("./App.tsx", { storage: first.storage() });
  for (const text of ["123", "NORMAL FLOAT DEGREE"]) expect(second.texts()).toContain(text);
});

test("choosing a skin restyles the calculator and is kept between sessions", async () => {
  const first = await open("./App.tsx");
  const colorStatus = first.spanStyle("text:NORMAL FLOAT RADIAN")?.color;
  await first.press("id:skin-classic");
  const classicStatus = first.spanStyle("text:NORMAL FLOAT RADIAN")?.color;
  expect(classicStatus).not.toBe(colorStatus);
  const second = await open("./App.tsx", { storage: first.storage() });
  await second.waitUntil(() => second.spanStyle("text:NORMAL FLOAT RADIAN")?.color === classicStatus);
});

test("every key of every skin is a key the calculator knows", async () => {
  for (const skin of SKINS) {
    const app = await open("./App.tsx");
    await app.press(`id:skin-${skin.id}`);
    for (const spec of keysOf(skin.keypad)) await app.press(`id:key-${spec.key}`);
  }
});
