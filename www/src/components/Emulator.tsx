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

  createEffect(() => {
    const buffer = currentRom()?.bytes;
    if (!buffer) return;

    try {
      chip8.reset();
      chip8.load_rom(buffer);
      clearErrors();
      setInternals(readInternals());
      setEmulationState("running");
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

      let tickOutput: number;
      try {
        tickOutput = chip8.tick(delta);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        addError(msg);
        setInternals(readInternals());
        setEmulationState("error");
        return;
      }
      setInternals(readInternals());
      const shouldDraw = tickOutput % 2;
      const soundActive = (tickOutput >> 1) % 2;

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
            width={chip8.screen_width() * scale()}
            height={chip8.screen_height() * scale()}
          />
        </div>
        <Controls />
        <StatusBar />
      </div>
    </div>
  );
}

function Controls() {
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
      <label class="toolbar-field">
        Speed
        <select
          class="toolbar-select"
          value={speed()}
          onChange={({ currentTarget }) =>
            setSpeed(SPEEDS[currentTarget.selectedIndex])
          }
        >
          <For each={SPEEDS}>
            {(s) => <option value={s}>{s}×</option>}
          </For>
        </select>
      </label>
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
