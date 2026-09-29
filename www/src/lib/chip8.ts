import {Chip8} from "chip8-wasm";
import {memory} from "chip8-wasm/chip8_wasm_bg.wasm";
import {codeAround, type Internals} from "./internals.ts";

export const chip8 = new Chip8();

export function getScreen(): Uint8Array {
  return new Uint8Array(memory.buffer, chip8.screen_ptr(), chip8.screen_len());
}

export function readInternals(): Internals {
  const pc = chip8.pc();
  const sp = chip8.sp();
  const registers = new Uint8Array(memory.buffer, chip8.registers_ptr(), chip8.registers_len());
  const stack = new Uint16Array(memory.buffer, chip8.stack_ptr(), chip8.stack_len());
  const ram = new Uint8Array(memory.buffer, chip8.memory_ptr(), chip8.memory_len());

  return {
    pc,
    index: chip8.index(),
    sp,
    delayTimer: chip8.delay_timer(),
    soundTimer: chip8.sound_timer(),
    registers: Array.from(registers),
    stack: Array.from(stack.subarray(0, sp)),
    code: codeAround(ram, pc, 2, 4),
  };
}
