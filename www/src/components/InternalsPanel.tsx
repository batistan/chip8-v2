import { For, Index, Show } from "solid-js";
import { disassemble } from "../lib/disasm";
import { hex } from "../lib/internals";
import {
  codeView,
  currentRom,
  emulationState,
  internals,
  setCodeView,
  speed,
  type CodeView,
} from "../state";

const CODE_VIEWS: readonly { value: CodeView; label: string }[] = [
  { value: "hex", label: "HEX" },
  { value: "asm", label: "ASM" },
];

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
      <div class="window-body internals-body">
        <div class="toolbar-field" role="radiogroup" aria-labelledby="code-view-label">
          <span id="code-view-label">Code</span>
          <div class="toolbar-group">
            <For each={CODE_VIEWS}>
              {(view) => (
                <label class="toolbar-toggle">
                  <input
                    type="radio"
                    name="code-view"
                    value={view.value}
                    checked={codeView() === view.value}
                    onChange={() => setCodeView(view.value)}
                  />
                  {view.label}
                </label>
              )}
            </For>
          </div>
        </div>
        <pre class="console">{renderTrace()}</pre>
      </div>
    </section>
  );
}

function Label(props: { text: string }) {
  return <span class="dim">{props.text}</span>;
}

function renderTrace() {
  return (
    <>
      <Label text="ROM   " />{currentRom()?.name ?? "(none)"}{"\n"}
      <Label text="STATE " />{emulationState().toUpperCase()}{"\n"}
      <Label text="SPEED " />{speed()}×{"\n"}
      <span class="dim">--</span>{"\n"}
      <Show
        when={internals()}
        fallback={<span class="dim">; load a ROM to inspect the CPU{"\n"}</span>}
      >
        {(cpu) => (
          <>
            <Label text="PC " />{`0x${hex(cpu().pc, 4)}  `}
            <Label text="I  " />{`0x${hex(cpu().index, 4)}\n`}
            <Label text="SP " />{`0x${hex(cpu().sp, 2)}    `}
            <Label text="DT " />{`${hex(cpu().delayTimer, 2)}  `}
            <Label text="ST " />{`${hex(cpu().soundTimer, 2)}\n`}
            <span class="dim">--</span>{"\n"}
            <Index each={cpu().registers}>
              {(value, i) => (
                <>
                  <Label text={`V${hex(i, 1)} `} />{hex(value(), 2)}
                  {i % 4 === 3 ? "\n" : "  "}
                </>
              )}
            </Index>
            <span class="dim">-- stack</span>{"\n"}
            <Show when={cpu().stack.length > 0} fallback={<span class="dim">(empty){"\n"}</span>}>
              <Index each={cpu().stack}>
                {(addr, i) => (
                  <>
                    <Label text={`${hex(i, 1)}: `} />{`0x${hex(addr(), 4)}\n`}
                  </>
                )}
              </Index>
            </Show>
            <span class="dim">-- code</span>{"\n"}
            <For each={cpu().code}>
              {(line) => (
                <span class={line.addr === cpu().pc ? "ok" : undefined}>
                  {`${line.addr === cpu().pc ? ">" : " "}0x${hex(line.addr, 4)}  `}
                  {codeView() === "asm" ? disassemble(line.opcode) : hex(line.opcode, 4)}
                  {"\n"}
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
