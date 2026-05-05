import type { Chip8 } from "chip8-wasm";
import { setPressedKeys } from "../state";

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

export const defaultKeyLabels: Record<string, string> = {
  Digit1: "1",
  Digit2: "2",
  Digit3: "3",
  Digit4: "4",
  KeyQ: "Q",
  KeyW: "W",
  KeyE: "E",
  KeyR: "R",
  KeyA: "A",
  KeyS: "S",
  KeyD: "D",
  KeyF: "F",
  KeyZ: "Z",
  KeyX: "X",
  KeyC: "C",
  KeyV: "V",
}

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
    setPressedKeys(prev => new Set(prev).add(key));
    chip8.key_down(key);
  } else {
    setPressedKeys(prev => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
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
