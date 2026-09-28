// @vitest-environment node
import { describe, expect, test } from "vitest";
import { disassemble } from "./disasm";

describe("disassemble", () => {
  test.each([
    [0x00e0, "CLS"],
    [0x00ee, "RET"],
    [0x1228, "JP 0x228"],
    [0x2abc, "CALL 0xABC"],
    [0x3a0f, "SE VA, 0x0F"],
    [0x4b10, "SNE VB, 0x10"],
    [0x5120, "SE V1, V2"],
    [0x6a02, "LD VA, 0x02"],
    [0x7bfe, "ADD VB, 0xFE"],
    [0x8120, "LD V1, V2"],
    [0x8121, "OR V1, V2"],
    [0x8122, "AND V1, V2"],
    [0x8123, "XOR V1, V2"],
    [0x8124, "ADD V1, V2"],
    [0x8125, "SUB V1, V2"],
    [0x8126, "SHR V1"],
    [0x8127, "SUBN V1, V2"],
    [0x812e, "SHL V1"],
    [0x9120, "SNE V1, V2"],
    [0xa22a, "LD I, 0x22A"],
    [0xb300, "JP V0, 0x300"],
    [0xc3ff, "RND V3, 0xFF"],
    [0xdab6, "DRW VA, VB, 6"],
    [0xe09e, "SKP V0"],
    [0xe0a1, "SKNP V0"],
    [0xf307, "LD V3, DT"],
    [0xf30a, "LD V3, K"],
    [0xf315, "LD DT, V3"],
    [0xf318, "LD ST, V3"],
    [0xf31e, "ADD I, V3"],
    [0xf329, "LD F, V3"],
    [0xf333, "LD B, V3"],
    [0xf355, "LD [I], V3"],
    [0xf365, "LD V3, [I]"],
  ])("%s → %s", (opcode, expected) => {
    expect(disassemble(opcode)).toBe(expected);
  });

  test.each([0x0123, 0x5121, 0x812f, 0x9121, 0xe0ff, 0xf0ff])(
    "falls back to a data word for unsupported %s",
    (opcode) => {
      expect(disassemble(opcode)).toMatch(/^DW 0x[0-9A-F]{4}$/);
    },
  );
});
