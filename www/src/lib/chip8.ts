import {Chip8} from "chip8-wasm";
import {memory} from "chip8-wasm/chip8_wasm_bg.wasm";

export const chip8 = new Chip8();

export function getScreen(): Uint8Array {
  return new Uint8Array(memory.buffer, chip8.screen_ptr(), chip8.screen_len());
}
