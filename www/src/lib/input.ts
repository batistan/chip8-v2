import type { Chip8 } from "chip8-wasm";

export const defaultKeyMap: Record<string, number> = {
  Digit1: 0x1,
  Digit2: 0x2,
  Digit3: 0x3,
  Digit4: 0xc,
  KeyQ: 0x4,
  KeyW: 0x5,
  KeyE: 0x6,
  KeyR: 0xd,
  KeyA: 0x7,
  KeyS: 0x8,
  KeyD: 0x9,
  KeyF: 0xe,
  KeyZ: 0xa,
  KeyX: 0x0,
  KeyC: 0xb,
  KeyV: 0xf,
};

type KeyboardEventType = "keydown" | "keyup";

function handleKeyPress(
  event: KeyboardEvent,
  action: KeyboardEventType,
  chip8: Chip8,
  keyMap: Record<string, number>
) {
  if (event.repeat) return;

  const key = keyMap[event.code];

  if (key === undefined) return;

  event.preventDefault();

  if (action === "keydown") {
    chip8.key_down(key);
  } else {
    chip8.key_up(key);
  }
}

export type CleanupInputCallback = () => void;

export function setupInput(
    chip8: Chip8,
    keyMap: Record<string, number> = defaultKeyMap
): CleanupInputCallback {
  const keyDown = (e: KeyboardEvent) => handleKeyPress(e, "keydown", chip8, keyMap);
  const keyUp = (e: KeyboardEvent) => handleKeyPress(e, "keyup", chip8, keyMap);

  window.addEventListener("keydown", keyDown);
  window.addEventListener("keyup", keyUp);

  return () => {
    window.removeEventListener("keydown", keyDown);
    window.removeEventListener("keyup", keyUp);
  };
}
