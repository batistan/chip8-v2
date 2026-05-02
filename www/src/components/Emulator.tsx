import {createSignal, onCleanup, onMount} from "solid-js";
import {updateScreen} from "../lib/renderer.ts";
import {chip8, getScreen} from "../lib/chip8.ts";
import { setupInput } from "../lib/input.ts";

export default function Emulator() {
  let canvasHolderRef!: HTMLDivElement;
  let canvasRef!: HTMLCanvasElement;

  const [screenWidth] = createSignal(chip8.screen_width());
  const [screenHeight] = createSignal(chip8.screen_height());

  const [scale, setScale] = createSignal(10);

  const resizeObserver = new ResizeObserver(([{ contentRect: { height, width }}] ) => {
    setScale(Math.floor(Math.min(height / screenHeight(), width / screenWidth())));
  })

  onMount(() => {
    const cleanupInput = setupInput(chip8);
    if (canvasHolderRef) {
      resizeObserver.observe(canvasHolderRef);
    }

    let animationId = 0;

    fetch("/roms/PONG.ch8")
      .then(r => r.arrayBuffer())
      .then(buffer => {
        chip8.load_rom(new Uint8Array(buffer))

        let lastTime: number | null = null;

        const animationCallback = ((time: number) => {
          if (lastTime === null) { lastTime = time; }

          const delta = time - lastTime;

          const tickOutput = chip8.tick(delta)
          const shouldDraw = tickOutput % 2;
          const soundActive = (tickOutput >> 1) % 2;

          if (shouldDraw) {
            updateScreen(
              canvasRef.getContext("2d")!,
              getScreen(),
              screenWidth(),
              scale()
            );
          }

          if (soundActive) {
            // TODO sound context
          }

          lastTime = time;
          animationId = requestAnimationFrame(animationCallback);
        });

        animationId = requestAnimationFrame(animationCallback);
      });

    onCleanup(() => {
      cleanupInput();
      cancelAnimationFrame(animationId)
      resizeObserver.disconnect();
    });
  })

  return <div ref={canvasHolderRef}>
    <canvas
      ref={canvasRef}
      id="emulator-canvas"
      width={screenWidth() * scale()}
      height={screenHeight() * scale()}
    />
  </div>
}