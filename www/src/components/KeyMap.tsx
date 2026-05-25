import { For, createSignal, onMount, type Setter } from "solid-js";
import { defaultKeyLabels, defaultKeyMap } from "../lib/input";
import { pressedKeys } from "../state";

export function KeyMap() {
  const [keyLabels, setKeyLabels] =
    createSignal<Record<string, string>>(defaultKeyLabels);

  onMount(async () => {
    await tryUpdateKeyMap(setKeyLabels);
  });

  return (
    <section class="window">
      <div class="window-titlebar">
        <span class="window-title">KEYMAP.CPL — Input</span>
        <div class="window-controls">
          <button type="button" class="window-control" aria-label="Minimize">_</button>
          <button type="button" class="window-control" aria-label="Maximize">▢</button>
          <button type="button" class="window-control" aria-label="Close">×</button>
        </div>
      </div>
      <div class="window-body">
        <div class="keymap-grid">
          <For each={Object.entries(keyLabels())}>
            {([code, label]) => {
              const c8Val = defaultKeyMap[code];
              return (
                <Key
                  primary={label}
                  secondary={`0x${c8Val.toString(16).toLocaleUpperCase()}`}
                  isSelected={pressedKeys().has(c8Val)}
                />
              );
            }}
          </For>
        </div>
        <div class="keymap-hint">QWERTY → CHIP-8 hex pad</div>
      </div>
    </section>
  );
}

interface KeyProps {
  primary: string;
  secondary: string | number;
  isSelected: boolean;
}

function Key(props: KeyProps) {
  return (
    <div class={"keycap" + (props.isSelected ? " pressed" : "")}>
      <span class="chip">{props.secondary}</span>
      <span class="phys">{props.primary.toLocaleUpperCase()}</span>
    </div>
  );
}

async function tryUpdateKeyMap(
  handleSetKeyMap: Setter<Record<string, string>>,
) {
  if (!navigator.keyboard) return;

  const layoutMap = await navigator.keyboard?.getLayoutMap()

  if (!layoutMap) return;

  for (const [keyCode, physical] of layoutMap) {
    if (keyCode in defaultKeyMap) {
      handleSetKeyMap((prev) => ({ ...prev, [keyCode]: physical }));
    }
  }
}
