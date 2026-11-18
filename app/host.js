// The page's side of the app's `haptic.tap` posts. Android browsers vibrate (`navigator.vibrate`); iOS Safari has no
// vibration API, but toggling a `<input type="checkbox" switch>` plays the system haptic there (Safari 18+).
let iosSwitch;

function toggleIosSwitch() {
  if (!iosSwitch) {
    iosSwitch = document.createElement("label");
    iosSwitch.ariaHidden = "true";
    iosSwitch.style.display = "none";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.setAttribute("switch", "");
    iosSwitch.append(input);
    document.body.append(iosSwitch);
  }
  iosSwitch.click();
}

function tap() {
  if (typeof navigator.vibrate === "function") navigator.vibrate(8);
  else if (typeof document !== "undefined") toggleIosSwitch();
}

export function createHost() {
  return {
    call(name) {
      throw new Error(`no host function ${name}`);
    },
    post(name) {
      if (name === "haptic.tap") tap();
    },
  };
}
