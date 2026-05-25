use chip8_core::{Emulator, EmulatorError, RandomSource, TickOutput};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

// uses a buffer to avoid needing to allocate a new array and call getrandom::fill every time
// TODO see if we can make buffer size dynamic
// probably not worth it since we're not doing anything with the buffer except reading it
// and occasionally filling it with new random bytes, so unlikely to be a bottleneck
struct WasmRng {
    buffer: [u8; 32],
    idx: usize,
}

impl WasmRng {
    fn new() -> Self {
        WasmRng {
            buffer: [0; 32],
            idx: 0,
        }
    }
}

impl RandomSource for WasmRng {
    fn next_u8(&mut self) -> u8 {
        if self.idx == self.buffer.len() {
            getrandom::fill(&mut self.buffer).unwrap();
            self.idx = 0;
            self.buffer[0]
        } else {
            self.idx += 1;
            self.buffer[self.idx - 1]
        }
    }
}

#[wasm_bindgen]
pub struct Chip8 {
    emulator: Emulator,
}

#[wasm_bindgen]
impl Chip8 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // idempotent; no need to check if already set
        console_error_panic_hook::set_once();

        Chip8 {
            emulator: Emulator::new(Box::new(WasmRng::new())),
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), JsError> {
        self.emulator.load_rom(data).map_err(to_js_error)
    }

    /// Returns a packed u32: low 16 bits = flags, where bit 0 = screen_updated, bit 1 = sound_active.
    /// On error, returns a sentinel or you use Result<u32, JsError>.
    pub fn tick(&mut self, delta_ms: f64) -> Result<u32, JsError> {
        let tick_output = self.emulator.tick(delta_ms).map_err(to_js_error)?;

        Ok(to_packed_num(tick_output))
    }

    pub fn key_down(&mut self, key: u8) {
        self.emulator.key_down(key);
    }

    pub fn key_up(&mut self, key: u8) {
        self.emulator.key_up(key);
    }

    pub fn screen_ptr(&self) -> *const u8 {
        self.emulator.screen().as_ptr()
    }

    pub fn screen_len(&self) -> usize {
        self.emulator.screen().len()
    }

    pub fn screen_width(&self) -> usize {
        self.emulator.screen_width()
    }

    pub fn screen_height(&self) -> usize {
        self.emulator.screen_height()
    }

    pub fn reset(&mut self) {
        self.emulator.reset();
    }

    // Internals accessors. These return copies so writes from JS to the
    // resulting typed arrays do not propagate back to the emulator state.
    pub fn registers(&self) -> Vec<u8> {
        self.emulator.registers().to_vec()
    }

    pub fn memory(&self) -> Vec<u8> {
        self.emulator.memory().to_vec()
    }

    pub fn stack(&self) -> Vec<u16> {
        self.emulator.stack().to_vec()
    }

    pub fn pc(&self) -> u16 {
        self.emulator.pc()
    }

    pub fn index(&self) -> u16 {
        self.emulator.index()
    }

    pub fn sp(&self) -> u8 {
        self.emulator.sp()
    }

    pub fn delay_timer(&self) -> u8 {
        self.emulator.delay_timer()
    }

    pub fn sound_timer(&self) -> u8 {
        self.emulator.sound_timer()
    }
}

fn to_packed_num(value: TickOutput) -> u32 {
    (value.screen_updated as u32) | ((value.sound_active as u32) << 1)
}

fn to_js_error(e: EmulatorError) -> JsError {
    JsError::new(&e.to_string())
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}
