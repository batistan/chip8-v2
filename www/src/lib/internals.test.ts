// @vitest-environment node
import { describe, expect, test } from "vitest";
import { codeAround, hex } from "./internals";

describe("hex", () => {
  test("pads to the requested width and uppercases", () => {
    expect(hex(0xa, 2)).toBe("0A");
    expect(hex(0x200, 4)).toBe("0200");
  });

  test("does not truncate values wider than the width", () => {
    expect(hex(0x1234, 2)).toBe("1234");
  });
});

describe("codeAround", () => {
  const memory = new Uint8Array([0x00, 0xe0, 0x6a, 0x02, 0xa2, 0x2a, 0x12, 0x00]);

  test("reads big-endian opcodes around pc", () => {
    expect(codeAround(memory, 2, 1, 1)).toEqual([
      { addr: 0, opcode: 0x00e0 },
      { addr: 2, opcode: 0x6a02 },
      { addr: 4, opcode: 0xa22a },
    ]);
  });

  test("skips addresses outside of memory", () => {
    expect(codeAround(memory, 0, 2, 0)).toEqual([{ addr: 0, opcode: 0x00e0 }]);
    expect(codeAround(memory, 6, 0, 2)).toEqual([{ addr: 6, opcode: 0x1200 }]);
  });
});
