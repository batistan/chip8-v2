export type PresetRom = {
  id: string,
  title: string,
  description?: string,
  path: string
}

export const PRESET_ROMS: readonly PresetRom[] = [
  { id: "pong", title: "Pong", description: "Classic two-player game", path: "/roms/PONG.ch8" }
]
