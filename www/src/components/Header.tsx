export function Header() {
  return (
    <header class="header">
      <div class="header-inner">
        <div class="brand">
          <span class="brand-mark" aria-hidden="true" />
          <span class="brand-name">CHIP-8 Emulator</span>
        </div>
        <nav class="menubar" aria-label="Application menu">
          <span class="menubar-item"><u>F</u>ile</span>
          <span class="menubar-item"><u>V</u>iew</span>
          <span class="menubar-item"><u>O</u>ptions</span>
          <span class="menubar-item"><u>H</u>elp</span>
        </nav>
      </div>
    </header>
  );
}
