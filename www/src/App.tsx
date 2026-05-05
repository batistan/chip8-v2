import { For } from "solid-js";
import Emulator from "./components/Emulator";
import { Header } from "./components/Header";
import { InternalsPanel } from "./components/InternalsPanel";
import { KeyMap } from "./components/KeyMap";
import { RomSelector } from "./components/RomSelector";
import { dismissError, getErrors } from "./state";

export function App() {
  return (
    <>
      <Header />
      <main class="main">
        <aside>
          <InternalsPanel />
        </aside>
        <div class="canvas-col">
          <Emulator />
          <ErrorPane />
          <RomSelector />
        </div>
        <aside>
          <KeyMap />
        </aside>
      </main>
    </>
  );
}

function ErrorPane() {
  return (
    <div class="error-list">
      <For each={getErrors()}>
        {(error) => (
          <button class="error-item" onClick={() => dismissError(error.id)}>
            {error.message}
          </button>
        )}
      </For>
    </div>
  );
}
