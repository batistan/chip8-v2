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
        <div class="center">
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
    <For each={getErrors()}>
      {(error) => (
        <button onClick={() => dismissError(error.id)}>{error.message}</button>
      )}
    </For>
  );
}
