export type CodeLine = { addr: number; opcode: number };

export type Internals = {
  pc: number;
  index: number;
  sp: number;
  delayTimer: number;
  soundTimer: number;
  registers: readonly number[];
  stack: readonly number[];
  code: readonly CodeLine[];
};

export function hex(value: number, width: number): string {
  return value.toString(16).toUpperCase().padStart(width, "0");
}

export function codeAround(
  memory: Uint8Array,
  pc: number,
  before: number,
  after: number,
): CodeLine[] {
  const lines: CodeLine[] = [];
  for (let i = -before; i <= after; i++) {
    const addr = pc + i * 2;
    if (addr < 0 || addr + 1 >= memory.length) continue;
    lines.push({ addr, opcode: (memory[addr] << 8) | memory[addr + 1] });
  }
  return lines;
}
