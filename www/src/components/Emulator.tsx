import { createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { updateScreen } from "../lib/renderer.ts";
import { chip8, getScreen } from "../lib/chip8.ts";
import { setupInput } from "../lib/input.ts";
import {
  addError,
  clearErrors,
  currentRom,
  emulationState,
  setCurrentRom,
  setEmulationState,
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
      if (lastTime === null) {
        lastTime = time;
      }

      const delta = time - lastTime;

      const tickOutput = chip8.tick(delta);
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
        <canvas
          ref={canvasRef}
          class="canvas-screen"
          width={chip8.screen_width() * scale()}
          height={chip8.screen_height() * scale()}
        />
      </div>
      <StatusBar />
    </div>
  );
}

// TODO localization
function StatusBar() {
  return (
    <div class="canvas-status">
      <div class="left">
        <span class="status-dot">{emulationState()}</span>
      </div>
      <div class="right"></div>
    </div>
  );
}
