export function updateScreen(
  ctx: CanvasRenderingContext2D,
  buffer: Uint8Array,
  screenWidth: number,
  scale: number
) {
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.fillStyle = getComputedStyle(ctx.canvas).getPropertyValue("--canvas-fg").trim()
  buffer.forEach((val, idx) => {
    if (val === 0) return;
    const x = idx % screenWidth;
    const y = Math.floor(idx / screenWidth);
    ctx.fillRect(x * scale, y * scale, scale, scale);
  })
}
