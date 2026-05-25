import { currentRom, emulationState } from "../state";

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
  const rom = currentRom()?.name ?? "(none)";
  const state = emulationState();
  return (
    <>
      <span class="dim">C:\CHIP8\&gt;</span> bootstrap{"\n"}
      <span class="ok">[OK]</span> WASM core attached{"\n"}
      <span class="ok">[OK]</span> 4 KiB RAM cleared{"\n"}
      <span class="ok">[OK]</span> Display 64×32 mono{"\n"}
      <span class="dim">--</span>{"\n"}
      ROM   : {rom}{"\n"}
      STATE : {state.toUpperCase()}{"\n"}
      PC    : 0x0200{"\n"}
      I     : 0x0000{"\n"}
      SP    : 0x00{"\n"}
      DT/ST : 00 / 00{"\n\n"}
      <span class="dim">; disassembly will appear here</span>{"\n"}
      <span class="dim">C:\CHIP8\&gt;</span><span class="cur"> </span>
    </>
  );
}
