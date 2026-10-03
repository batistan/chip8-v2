import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import type { Chip8 } from "chip8-wasm";
import { isModalOpen, modalClosed, modalOpened } from "../state";
import { defaultKeyMap, setupInput, type CleanupInputCallback } from "./input";

function chip8Fake() {
  return {
    key_down: vi.fn(),
    key_up: vi.fn(),
  };
}

type FakeChip8 = ReturnType<typeof chip8Fake>;

function asChip8(fake: FakeChip8): Chip8 {
  return fake as unknown as Chip8;
}

describe("keyMapping", () => {
  test("matches the standard CHIP-8 ↔ QWERTY layout", () => {
    expect(defaultKeyMap).toEqual({
      Digit1: 0x1, Digit2: 0x2, Digit3: 0x3, Digit4: 0xc,
      KeyQ:   0x4, KeyW:   0x5, KeyE:   0x6, KeyR:   0xd,
      KeyA:   0x7, KeyS:   0x8, KeyD:   0x9, KeyF:   0xe,
      KeyZ:   0xa, KeyX:   0x0, KeyC:   0xb, KeyV:   0xf,
    });
  });
});

describe("setupInput", () => {
  let chip8: FakeChip8;
  let cleanup: CleanupInputCallback | undefined;

  beforeEach(() => {
    chip8 = chip8Fake();
  });

  afterEach(() => {
    cleanup?.();
    cleanup = undefined;
  });

  test("keydown on a mapped code calls key_down with the mapped value", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyA" }));

    expect(chip8.key_down).toHaveBeenCalledOnce();
    expect(chip8.key_down).toHaveBeenCalledWith(0x5);
    expect(chip8.key_up).not.toHaveBeenCalled();
  });

  test("keyup on a mapped code calls key_up with the mapped value", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    window.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyA" }));

    expect(chip8.key_up).toHaveBeenCalledOnce();
    expect(chip8.key_up).toHaveBeenCalledWith(0x5);
    expect(chip8.key_down).not.toHaveBeenCalled();
  });

  test("a code that maps to 0 still dispatches (falsy-zero regression)", () => {
    cleanup = setupInput(asChip8(chip8), { KeyZeroTest: 0x0 });

    window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyZeroTest" }));
    window.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyZeroTest" }));

    expect(chip8.key_down).toHaveBeenCalledWith(0x0);
    expect(chip8.key_up).toHaveBeenCalledWith(0x0);
  });

  test("repeat keydown events are ignored", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    window.dispatchEvent(
      new KeyboardEvent("keydown", { code: "KeyA", repeat: true }),
    );

    expect(chip8.key_down).not.toHaveBeenCalled();
  });

  test("unmapped codes are ignored", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyB" }));
    window.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyB" }));

    expect(chip8.key_down).not.toHaveBeenCalled();
    expect(chip8.key_up).not.toHaveBeenCalled();
  });

  test("preventDefault is called on matched keys", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    const event = new KeyboardEvent("keydown", {
      code: "KeyA",
      cancelable: true,
    });
    window.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
  });

  test("preventDefault is not called on unmapped keys", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

    const event = new KeyboardEvent("keydown", {
      code: "KeyB",
      cancelable: true,
    });
    window.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(false);
  });

  describe("while a modal is open", () => {
    afterEach(() => {
      while (isModalOpen()) modalClosed();
    });

    test("keydown is ignored and not prevented", () => {
      cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });
      modalOpened();

      const event = new KeyboardEvent("keydown", {
        code: "KeyA",
        cancelable: true,
      });
      window.dispatchEvent(event);

      expect(chip8.key_down).not.toHaveBeenCalled();
      expect(event.defaultPrevented).toBe(false);
    });

    test("keyup still releases a key held before the modal opened", () => {
      cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });

      window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyA" }));
      modalOpened();
      window.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyA" }));

      expect(chip8.key_up).toHaveBeenCalledWith(0x5);
    });

    test("keydown works again once the modal closes", () => {
      cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });
      modalOpened();
      modalClosed();

      window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyA" }));

      expect(chip8.key_down).toHaveBeenCalledWith(0x5);
    });
  });

  test("cleanup removes both keydown and keyup listeners", () => {
    cleanup = setupInput(asChip8(chip8), { KeyA: 0x5 });
    cleanup();
    cleanup = undefined;

    window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyA" }));
    window.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyA" }));

    expect(chip8.key_down).not.toHaveBeenCalled();
    expect(chip8.key_up).not.toHaveBeenCalled();
  });
});
