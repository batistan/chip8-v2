import { createSignal } from "solid-js";
import { Dialog } from "./Dialog";

export const REPO_URL = "https://github.com/batistan/chip8-v2";

export function Header() {
  const [aboutOpen, setAboutOpen] = createSignal(false);
  const [settingsOpen, setSettingsOpen] = createSignal(false);

  return (
    <header class="header">
      <div class="header-inner">
        <div class="brand">
          <span class="brand-mark" aria-hidden="true" />
          <span class="brand-name">CHIP-8 Emulator</span>
        </div>
        <nav class="menubar" aria-label="Application menu">
          <button
            type="button"
            class="menubar-item"
            aria-haspopup="dialog"
            onClick={() => setAboutOpen(true)}
          >
            About
          </button>
          <a
            class="menubar-item"
            href={REPO_URL}
            target="_blank"
            rel="noopener noreferrer"
          >
            GitHub
          </a>
          <button
            type="button"
            class="menubar-item"
            aria-haspopup="dialog"
            onClick={() => setSettingsOpen(true)}
          >
            Settings
          </button>
        </nav>
      </div>

      <Dialog
        title="About CHIP-8 Emulator"
        open={aboutOpen()}
        onClose={() => setAboutOpen(false)}
      >
        <AboutContent onClose={() => setAboutOpen(false)} />
      </Dialog>

      <Dialog
        title="Settings"
        open={settingsOpen()}
        onClose={() => setSettingsOpen(false)}
      >
        <div class="dialog-actions">
          <button
            type="button"
            class="toolbar-button"
            onClick={() => setSettingsOpen(false)}
          >
            Close
          </button>
        </div>
      </Dialog>
    </header>
  );
}

function AboutContent(props: { onClose: () => void }) {
  return (
    <>
      <p>
        A CHIP-8 emulator written in Rust, compiled to WebAssembly, and running
        right here in your browser. CHIP-8 is a tiny virtual machine from the
        1970s, originally used to write games for hobbyist microcomputers.
      </p>

      <h2 class="dialog-heading">How to use it</h2>
      <ul class="dialog-list">
        <li>
          <strong>Load a game:</strong> pick one from the Cartridge Library, or
          click <em>Browse…</em> to load your own <code>.ch8</code> file.
        </li>
        <li>
          <strong>Play:</strong> the CHIP-8 has a 16-key hex keypad, mapped to
          the <code>1234</code> / <code>QWER</code> / <code>ASDF</code> /{" "}
          <code>ZXCV</code> block on your keyboard. The Keymap panel shows
          which key is which and lights up as you press them.
        </li>
        <li>
          <strong>Control emulation:</strong> use the toolbar under the screen
          to pause, step one instruction at a time while paused, or change the
          speed.
        </li>
        <li>
          <strong>Look inside:</strong> the Internals panel shows the CPU's
          registers, timers, and stack as the program runs, with the code
          around the program counter as hex or disassembly.
        </li>
      </ul>

      <p>
        Source code and docs are on{" "}
        <a href={REPO_URL} target="_blank" rel="noopener noreferrer">
          GitHub
        </a>
        .
      </p>

      <div class="dialog-actions">
        <button type="button" class="toolbar-button" onClick={() => props.onClose()}>
          OK
        </button>
      </div>
    </>
  );
}
