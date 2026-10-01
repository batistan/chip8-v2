import { For, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { updateScreen } from "../lib/renderer.ts";
import { chip8, getScreen, readInternals } from "../lib/chip8.ts";
import { setupInput } from "../lib/input.ts";
import {
  addError,
  clearErrors,
  currentRom,
  emulationState,
  setCurrentRom,
  setEmulationState,
  setInternals,
  setSpeed,
  speed,
  SPEEDS,
  togglePause,
} from "../state.ts";

export default function Emulator() {
  let canvasHolderRef!: HTMLDivElement;
  let canvasRef!: HTMLCanvasElement;

  const [scale, setScale] = createSignal(10);

  const resizeObserver = new ResizeObserver(
    ([
      {
        contentRect: { height, width },
      },
    ]) => {
      setScale(
        Math.floor(
          Math.min(
            height / chip8.screen_height(),
            width / chip8.screen_width(),
          ),
        ),
      );
    },
  );

  onMount(() => {
    const cleanupInput = setupInput(chip8);
    if (canvasHolderRef) {
      resizeObserver.observe(canvasHolderRef);
    }

    onCleanup(() => {
      cleanupInput();
      resizeObserver.disconnect();
    });
  });

  // runs a tick/step, then syncs the canvas and internals; false on emulator error
  function runCpu(run: () => number): boolean {
    let output: number;
    try {
      output = run();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addError(msg);
      setInternals(readInternals());
      setEmulationState("error");
      return false;
    }
    setInternals(readInternals());

    const shouldDraw = output & 1;
    const soundActive = output & 2;

    if (shouldDraw) {
      updateScreen(
        canvasRef.getContext("2d")!,
        getScreen(),
        chip8.screen_width(),
        scale(),
      );
    }

    if (soundActive) {
      // TODO sound context
    }

    return true;
  }

  createEffect(() => {
    const buffer = currentRom()?.bytes;
    if (!buffer) return;

    try {
      chip8.reset();
      chip8.load_rom(buffer);
      clearErrors();
      setInternals(readInternals());
      setEmulationState("running");
      // keeps Space/Enter from re-activating the ROM button that loaded this game
      canvasRef.focus();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addError(msg);
      setEmulationState("error");
      setCurrentRom(null);
      return;
    }

    let animationId = 0;
    let lastTime: number | null = null;

    const animationCallback = (time: number) => {
      if (lastTime === null || emulationState() !== "running") {
        lastTime = time;
        animationId = requestAnimationFrame(animationCallback);
        return;
      }

      const delta = (time - lastTime) * speed();
      if (!runCpu(() => chip8.tick(delta))) return;

      lastTime = time;
      animationId = requestAnimationFrame(animationCallback);
    };

    animationId = requestAnimationFrame(animationCallback);

    onCleanup(() => {
      cancelAnimationFrame(animationId);
    });
  });

  return (
    <div ref={canvasHolderRef} class="canvas-col">
      <div class="canvas-frame">
        <div class="canvas-screen-wrap">
          <canvas
            ref={canvasRef}
            class="canvas-screen"
            tabIndex={-1}
            role="img"
            aria-label="CHIP-8 display"
            width={chip8.screen_width() * scale()}
            height={chip8.screen_height() * scale()}
          />
        </div>
        <Controls onStep={() => runCpu(() => chip8.step())} />
        <StatusBar />
      </div>
    </div>
  );
}

function Controls(props: { onStep: () => void }) {
  const canToggle = () =>
    emulationState() === "running" || emulationState() === "paused";

  return (
    <div class="toolbar" role="toolbar" aria-label="Emulation controls">
      <button
        type="button"
        class="toolbar-button"
        disabled={!canToggle()}
        aria-pressed={emulationState() === "paused"}
        onClick={togglePause}
      >
        {emulationState() === "paused" ? "▶ Resume" : "❚❚ Pause"}
      </button>
      <button
        type="button"
        class="toolbar-button"
        disabled={emulationState() !== "paused"}
        title="Execute one instruction"
        onClick={() => props.onStep()}
      >
        ▶| Step
      </button>
      <div class="toolbar-field" role="radiogroup" aria-labelledby="speed-label">
        <span id="speed-label">Speed</span>
        <div class="toolbar-group">
          <For each={SPEEDS}>
            {(s) => (
              <label class="toolbar-toggle">
                <input
                  type="radio"
                  name="speed"
                  value={s}
                  checked={speed() === s}
                  onChange={() => setSpeed(s)}
                />
                {s}×
              </label>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}

// TODO localization
// emulation state e.g. stopped should map to Stopped in the display, or some other string in other locales
function StatusBar() {
  return (
    <div class="statusbar" role="status">
      <span class="statusbar-cell">
        <span class={`status-dot ${emulationState()}`}>{emulationState()}</span>
      </span>
      <span class="statusbar-cell grow">
        ROM: {currentRom()?.name ?? "—"}
      </span>
      <span class="statusbar-cell">64×32</span>
      <span class="statusbar-cell">CHIP-8</span>
    </div>
  );
}
