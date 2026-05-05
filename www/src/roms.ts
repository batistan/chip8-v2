export type PresetRom = {
  id: string,
  title: string,
  description?: string,
  path: string
}

export const PRESET_ROMS: readonly PresetRom[] = [
  { id: "15puzzle", title: "15 Puzzle", description: "Sliding tile puzzle", path: "/roms/15PUZZLE.ch8" },
  { id: "blinky", title: "Blinky", description: "Pac-Man clone", path: "/roms/BLINKY.ch8" },
  { id: "blitz", title: "Blitz", description: "Bomb the buildings before you crash", path: "/roms/BLITZ.ch8" },
  { id: "brix", title: "Brix", description: "Break all the bricks with a bouncing ball", path: "/roms/BRIX.ch8" },
  { id: "connect4", title: "Connect 4", path: "/roms/CONNECT4.ch8" },
  { id: "guess", title: "Guess", description: "Pick a number, the computer figures it out", path: "/roms/GUESS.ch8" },
  { id: "hidden", title: "Hidden", description: "Memory match card game", path: "/roms/HIDDEN.ch8" },
  { id: "invaders", title: "Invaders", description: "Space Invaders clone", path: "/roms/INVADERS.ch8" },
  { id: "kaleid", title: "Kaleidoscope", description: "Draw symmetric patterns", path: "/roms/KALEID.ch8" },
  { id: "life", title: "Game of Life", description: "Conway's cellular automaton", path: "/roms/LIFE.ch8" },
  { id: "maze", title: "Maze", description: "Random maze generator demo", path: "/roms/MAZE.ch8" },
  { id: "merlin", title: "Merlin", description: "Simon-style memory game", path: "/roms/MERLIN.ch8" },
  { id: "missile", title: "Missile", description: "Shoot incoming missiles before they land", path: "/roms/MISSILE.ch8" },
  { id: "pong", title: "Pong", description: "Classic two-player paddle game", path: "/roms/PONG.ch8" },
  { id: "pong2", title: "Pong 2", description: "Improved version of Pong", path: "/roms/PONG2.ch8" },
  { id: "puzzle", title: "Puzzle", description: "Sliding number puzzle", path: "/roms/PUZZLE.ch8" },
  { id: "syzygy", title: "Syzygy", description: "Snake-style game, eat without crashing", path: "/roms/SYZYGY.ch8" },
  { id: "tank", title: "Tank", description: "Hit targets with your tank without getting hit", path: "/roms/TANK.ch8" },
  { id: "tetris", title: "Tetris", path: "/roms/TETRIS.ch8" },
  { id: "tictac", title: "Tic-Tac-Toe", path: "/roms/TICTAC.ch8" },
  { id: "ufo", title: "UFO", description: "Shoot UFOs as they fly past", path: "/roms/UFO.ch8" },
  { id: "vbrix", title: "V-Brix", description: "Vertical Breakout variant", path: "/roms/VBRIX.ch8" },
  { id: "vers", title: "Vers", description: "Tron light cycles for two players", path: "/roms/VERS.ch8" },
  { id: "wipeoff", title: "Wipe-Off", description: "Wipe off all the bricks", path: "/roms/WIPEOFF.ch8" },
]
