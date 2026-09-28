type Fields = { x: string; y: string; n: string; kk: string; nnn: string };
type Entry = { mask: number; match: number; format: (f: Fields) => string };

const hex = (value: number, width: number) =>
  "0x" + value.toString(16).toUpperCase().padStart(width, "0");

const OPCODES: readonly Entry[] = [
  { mask: 0xffff, match: 0x00e0, format: () => "CLS" },
  { mask: 0xffff, match: 0x00ee, format: () => "RET" },
  { mask: 0xf000, match: 0x1000, format: (f) => `JP ${f.nnn}` },
  { mask: 0xf000, match: 0x2000, format: (f) => `CALL ${f.nnn}` },
  { mask: 0xf000, match: 0x3000, format: (f) => `SE ${f.x}, ${f.kk}` },
  { mask: 0xf000, match: 0x4000, format: (f) => `SNE ${f.x}, ${f.kk}` },
  { mask: 0xf00f, match: 0x5000, format: (f) => `SE ${f.x}, ${f.y}` },
  { mask: 0xf000, match: 0x6000, format: (f) => `LD ${f.x}, ${f.kk}` },
  { mask: 0xf000, match: 0x7000, format: (f) => `ADD ${f.x}, ${f.kk}` },
  { mask: 0xf00f, match: 0x8000, format: (f) => `LD ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8001, format: (f) => `OR ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8002, format: (f) => `AND ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8003, format: (f) => `XOR ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8004, format: (f) => `ADD ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8005, format: (f) => `SUB ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x8006, format: (f) => `SHR ${f.x}` },
  { mask: 0xf00f, match: 0x8007, format: (f) => `SUBN ${f.x}, ${f.y}` },
  { mask: 0xf00f, match: 0x800e, format: (f) => `SHL ${f.x}` },
  { mask: 0xf00f, match: 0x9000, format: (f) => `SNE ${f.x}, ${f.y}` },
  { mask: 0xf000, match: 0xa000, format: (f) => `LD I, ${f.nnn}` },
  { mask: 0xf000, match: 0xb000, format: (f) => `JP V0, ${f.nnn}` },
  { mask: 0xf000, match: 0xc000, format: (f) => `RND ${f.x}, ${f.kk}` },
  { mask: 0xf000, match: 0xd000, format: (f) => `DRW ${f.x}, ${f.y}, ${f.n}` },
  { mask: 0xf0ff, match: 0xe09e, format: (f) => `SKP ${f.x}` },
  { mask: 0xf0ff, match: 0xe0a1, format: (f) => `SKNP ${f.x}` },
  { mask: 0xf0ff, match: 0xf007, format: (f) => `LD ${f.x}, DT` },
  { mask: 0xf0ff, match: 0xf00a, format: (f) => `LD ${f.x}, K` },
  { mask: 0xf0ff, match: 0xf015, format: (f) => `LD DT, ${f.x}` },
  { mask: 0xf0ff, match: 0xf018, format: (f) => `LD ST, ${f.x}` },
  { mask: 0xf0ff, match: 0xf01e, format: (f) => `ADD I, ${f.x}` },
  { mask: 0xf0ff, match: 0xf029, format: (f) => `LD F, ${f.x}` },
  { mask: 0xf0ff, match: 0xf033, format: (f) => `LD B, ${f.x}` },
  { mask: 0xf0ff, match: 0xf055, format: (f) => `LD [I], ${f.x}` },
  { mask: 0xf0ff, match: 0xf065, format: (f) => `LD ${f.x}, [I]` },
];

export function disassemble(opcode: number): string {
  const entry = OPCODES.find((e) => (opcode & e.mask) === e.match);
  if (!entry) return `DW ${hex(opcode, 4)}`;

  return entry.format({
    x: `V${((opcode >> 8) & 0xf).toString(16).toUpperCase()}`,
    y: `V${((opcode >> 4) & 0xf).toString(16).toUpperCase()}`,
    n: String(opcode & 0xf),
    kk: hex(opcode & 0xff, 2),
    nnn: hex(opcode & 0xfff, 3),
  });
}
