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
    <div class="roms">
      <label class="upload-btn">
        Upload ROM
        <input
          id="uploadedRom"
          type="file"
          accept=".ch8,.c8"
          multiple={false}
          onChange={({ target }) => handleUpload(target.files?.[0])}
        />
      </label>
      <div class="rom-grid">
        <For each={PRESET_ROMS}>
          {(rom) => <RomCard rom={rom} handleLoad={handleLoad} isSelected={currentRom()?.name === rom.title} />}
        </For>
      </div>
    </div>
  );
}

interface RomCardProps {
  rom: PresetRom;
  handleLoad: (r: PresetRom) => void;
  isSelected: boolean;
}

function RomCard(props: RomCardProps) {
  return (
    <div class={"rom" + (props.isSelected ? " selected" : "")} onClick={() => props.handleLoad(props.rom)}>
      <h6 class="panel-header">{props.rom.title}</h6>
      <Show when={props.rom.description !== undefined}>
        <p>{props.rom.description}</p>
      </Show>
    </div>
  );
}
