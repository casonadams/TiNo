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

test("MATH shows its tabs, scrolls to the highlighted item, and an item's key types it", async () => {
  const app = await open("./App.tsx");
  await app.press("id:key-math");
  for (const text of ["MATH", "NUM", "PROB"]) expect(app.texts()).toContain(text);
  expect(app.shows("label:1:►Frac")).toBe(true);
  expect(app.shows("label:A:logBASE(")).toBe(false);
  expect(app.texts()).toContain("9↓");
  await app.press("id:key-up");
  expect(app.shows("label:A:logBASE(")).toBe(true);
  expect(app.shows("label:1:►Frac")).toBe(false);
  await app.press("id:key-right");
  expect(app.shows("label:9:gcd(")).toBe(true);
  await app.press("id:key-1");
  expect(app.shows("id:menu")).toBe(false);
  expect(app.shows("label:abs(")).toBe(true);
});

test("an error offers Quit and Goto; Goto returns to the entry", async () => {
  const app = await open("./App.tsx");
  for (const key of ["1", "divide", "0", "enter"]) await app.press(`id:key-${key}`);
  expect(app.texts()).toContain("ERR:DIVIDE BY 0");
  for (const label of ["1:Quit", "2:Goto"]) expect(app.shows(`label:${label}`)).toBe(true);
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
  for (const text of [".67", "NORMAL FIX2 REAL RADIAN"]) expect(app.texts()).toContain(text);
});

test("a function typed in Y= graphs, and TRACE reports its points", async () => {
  const app = await open("./App.tsx");
  for (const key of ["y-equals", "xt-theta-n", "square", "trace"]) await app.press(`id:key-${key}`);
  expect(app.shows("id:graph")).toBe(true);
  for (const text of ["Y1=X²", "X=0", "Y=0"]) expect(app.texts()).toContain(text);
  await app.press("id:key-right");
  expect(app.texts()).not.toContain("X=0");
});

test("2nd CALC asks for points on the graph and shows the result", async () => {
  const app = await open("./App.tsx");
  for (const key of ["y-equals", "xt-theta-n", "square", "second", "trace"]) await app.press(`id:key-${key}`);
  expect(app.shows("label:1:value")).toBe(true);
  await app.press("id:key-6");
  expect(app.texts()).toContain("Y1=X²");
  await app.press("id:key-right");
  await app.press("id:key-enter");
  expect(app.texts().some((text) => text.startsWith("dy/dx="))).toBe(true);
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

test("letters typed on the computer keyboard come out as the calculator's letters, ALPHA lock or not", async () => {
  const app = await open("./App.tsx");
  for (const keystroke of ["d", "shift+x", "2"]) await app.keyboard.press(keystroke);
  expect(app.shows("label:DX2")).toBe(true);
  await app.press("id:key-second");
  await app.press("id:key-alpha");
  for (const keystroke of ["a", "b"]) await app.keyboard.press(keystroke);
  expect(app.shows("label:DX2AB")).toBe(true);
});

test("the calculator is kept between sessions", async () => {
  const first = await open("./App.tsx");
  for (const key of ["1", "2", "3", "enter", "mode", "down", "down", "right", "enter", "second", "mode"]) {
    await first.press(`id:key-${key}`);
  }
  const second = await open("./App.tsx", { storage: first.storage() });
  for (const text of ["123", "NORMAL FLOAT REAL DEGREE"]) expect(second.texts()).toContain(text);
});

test("ON opens settings, where a skin restyles the calculator and is kept between sessions", async () => {
  const first = await open("./App.tsx");
  const colorStatus = first.spanStyle("text:NORMAL FLOAT REAL RADIAN")?.color;
  await first.press("id:key-on");
  expect(first.shows("id:settings")).toBe(true);
  await first.press("id:skin-classic");
  await first.press("id:key-on");
  expect(first.shows("id:settings")).toBe(false);
  const classicStatus = first.spanStyle("text:NORMAL FLOAT REAL RADIAN")?.color;
  expect(classicStatus).not.toBe(colorStatus);
  const second = await open("./App.tsx", { storage: first.storage() });
  await second.waitUntil(() => second.spanStyle("text:NORMAL FLOAT REAL RADIAN")?.color === classicStatus);
});

test("arrows on the settings screen change the skin and ENTER returns", async () => {
  const app = await open("./App.tsx");
  const colorStatus = app.spanStyle("text:NORMAL FLOAT REAL RADIAN")?.color;
  for (const key of ["on", "right", "enter"]) await app.press(`id:key-${key}`);
  expect(app.shows("id:settings")).toBe(false);
  expect(app.spanStyle("text:NORMAL FLOAT REAL RADIAN")?.color).not.toBe(colorStatus);
});

test("key haptics can be turned off on the settings screen and stay off", async () => {
  const first = await open("./App.tsx");
  for (const key of ["on", "down", "right", "enter"]) await first.press(`id:key-${key}`);
  const second = await open("./App.tsx", { storage: first.storage() });
  await second.press("id:key-on");
  const chosen = second.spanStyle("text:Color")?.color;
  await second.waitUntil(() => second.spanStyle("text:Off")?.color === chosen);
  expect(second.spanStyle("text:On")?.color).not.toBe(chosen);
});

test("a long program runs on between screen updates, and ON breaks one that never ends", async () => {
  const app = await open("./App.tsx");
  const keys = async (...ids: string[]) => {
    for (const id of ids) await app.press(`id:key-${id}`);
  };
  await keys("prgm", "left", "enter", "math", "enter");
  await keys("prgm", "4", "xt-theta-n", "comma", "1", "comma", "3", "0", "0", "0", "enter");
  await keys("prgm", "7", "enter", "prgm", "right", "3", "xt-theta-n", "second", "mode");
  await keys("prgm", "1", "enter");
  await app.waitUntil(() => app.texts().includes("Done"));
  expect(app.texts()).toContain("3001");
  await keys("prgm", "left", "enter", "apps", "enter", "prgm", "5", "1", "enter", "prgm", "7", "second", "mode");
  await keys("prgm", "2", "enter", "on");
  expect(app.texts()).toContain("ERR:BREAK");
  expect(app.shows("id:settings")).toBe(false);
});

test("every key of every skin but ON, which opens settings, is a key the calculator knows", async () => {
  for (const skin of SKINS) {
    const app = await open("./App.tsx");
    for (const target of ["id:key-on", `id:skin-${skin.id}`, "id:key-on"]) await app.press(target);
    for (const spec of keysOf(skin.keypad).filter((spec) => spec.key !== "on")) await app.press(`id:key-${spec.key}`);
  }
});
