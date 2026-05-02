import Emulator from "./components/Emulator";
import { Header } from "./components/Header";
import { InternalsPanel } from "./components/InternalsPanel";
import { KeyMap } from "./components/KeyMap";
import { RomSelector } from "./components/RomSelector";

export function App() {
  return <>
    <Header />
    <main class="main">
      <aside><InternalsPanel /></aside>
      <div class="center">
        <Emulator />
        <RomSelector />
      </div>
      <aside><KeyMap /></aside>
    </main>
  </>
}