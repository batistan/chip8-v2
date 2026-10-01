import { For, Show } from "solid-js";
import { PRESET_ROMS, type PresetRom } from "../roms";
import { addError, currentRom, setCurrentRom, setEmulationState } from "../state";

export function RomSelector() {
  async function handleUpload(file?: File) {
    if (!file) return;

    setCurrentRom({
      name: file.name,
      bytes: await file.arrayBuffer().then((b) => new Uint8Array(b)),
    });
  }

  async function handleLoad(rom: PresetRom) {
    await fetch(rom.path)
      .then((r) => r.arrayBuffer())
      .then((b) => new Uint8Array(b))
      .then((arr) =>
        setCurrentRom({
          name: rom.title,
          bytes: arr,
        })
      )
      .catch((e) => {
        const msg = e instanceof Error ? e.message : String(e);
        addError(msg);
        setEmulationState("error");
      });
  }

  return (
    <section class="window">
      <div class="window-titlebar">
        <span class="window-title">ROMs.LIB — Cartridge Library</span>
        <div class="window-controls">
          <button type="button" class="window-control" aria-label="Minimize">_</button>
          <button type="button" class="window-control" aria-label="Maximize">▢</button>
          <button type="button" class="window-control" aria-label="Close">×</button>
        </div>
      </div>
      <div class="window-body roms-body">
        <label class="upload">
          <div class="upload-left">
            <div class="upload-icon" aria-hidden="true">⌬</div>
            <div class="upload-text">
              <div class="title">Insert cartridge</div>
              <div class="sub">Drop or pick a .ch8 / .c8 file</div>
            </div>
          </div>
          <span class="upload-cta">Browse…</span>
          <input
            id="uploadedRom"
            type="file"
            accept=".ch8,.c8"
            multiple={false}
            onChange={({ target }) => handleUpload(target.files?.[0])}
          />
        </label>
        <div>
          <div class="rom-listing-label">Installed cartridges</div>
          <div class="rom-listing">
            <div class="rom-grid">
              <For each={PRESET_ROMS}>
                {(rom) => (
                  <RomCard
                    rom={rom}
                    handleLoad={handleLoad}
                    isSelected={currentRom()?.name === rom.title}
                  />
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

interface RomCardProps {
  rom: PresetRom;
  handleLoad: (r: PresetRom) => void;
  isSelected: boolean;
}

function RomCard(props: RomCardProps) {
  return (
    <button
      type="button"
      class={"rom-item" + (props.isSelected ? " active" : "")}
      aria-current={props.isSelected ? "true" : undefined}
      onClick={() => props.handleLoad(props.rom)}
    >
      <span class="rom-title">{props.rom.title}</span>
      <Show when={props.rom.description !== undefined}>
        <span class="rom-desc">— {props.rom.description}</span>
      </Show>
    </button>
  );
}
