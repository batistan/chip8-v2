import { For, Index, Show } from "solid-js";
import { hex } from "../lib/internals";
import { currentRom, emulationState, internals, speed } from "../state";

export function InternalsPanel() {
  return (
    <section class="window">
      <div class="window-titlebar">
        <span class="window-title">CPU.SYS — Internals</span>
        <div class="window-controls">
          <button type="button" class="window-control" aria-label="Minimize">_</button>
          <button type="button" class="window-control" aria-label="Maximize">▢</button>
          <button type="button" class="window-control" aria-label="Close">×</button>
        </div>
      </div>
      <div class="window-body">
        <pre class="console">{renderTrace()}</pre>
      </div>
    </section>
  );
}

function renderTrace() {
  return (
    <>
      {"ROM   : "}{currentRom()?.name ?? "(none)"}{"\n"}
      {"STATE : "}{emulationState().toUpperCase()}{"\n"}
      {"SPEED : "}{speed()}×{"\n"}
      <span class="dim">--</span>{"\n"}
      <Show
        when={internals()}
        fallback={<span class="dim">; load a ROM to inspect the CPU{"\n"}</span>}
      >
        {(cpu) => (
          <>
            {`PC 0x${hex(cpu().pc, 4)}  I  0x${hex(cpu().index, 4)}\n`}
            {`SP 0x${hex(cpu().sp, 2)}    DT ${hex(cpu().delayTimer, 2)}  ST ${hex(cpu().soundTimer, 2)}\n`}
            <span class="dim">--</span>{"\n"}
            <Index each={cpu().registers}>
              {(value, i) => (
                <>
                  <span class="dim">V{hex(i, 1)}</span> {hex(value(), 2)}
                  {i % 4 === 3 ? "\n" : "  "}
                </>
              )}
            </Index>
            <span class="dim">-- stack</span>{"\n"}
            <Show when={cpu().stack.length > 0} fallback={<span class="dim">(empty){"\n"}</span>}>
              <Index each={cpu().stack}>
                {(addr, i) => (
                  <>
                    {hex(i, 1)}: 0x{hex(addr(), 4)}{"\n"}
                  </>
                )}
              </Index>
            </Show>
            <span class="dim">-- code</span>{"\n"}
            <For each={cpu().code}>
              {(line) => (
                <span class={line.addr === cpu().pc ? "ok" : undefined}>
                  {`${line.addr === cpu().pc ? ">" : " "}0x${hex(line.addr, 4)}  ${hex(line.opcode, 4)}\n`}
                </span>
              )}
            </For>
          </>
        )}
      </Show>
      <span class="dim">C:\CHIP8\&gt;</span><span class="cur"> </span>
    </>
  );
}
