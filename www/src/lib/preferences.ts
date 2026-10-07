interface UserPrefs {
  isMuted: boolean,
  beepWaveform: OscillatorType,
  beepFrequencyHertz: number
}

const defaultPrefs: UserPrefs = {
  isMuted: false,
  beepWaveform: "square",
  beepFrequencyHertz: 440
}

export function getUserPrefs(): UserPrefs {
  if (!localStorage) return defaultPrefs;

  const saved = localStorage.getItem("prefs")

  return saved ? JSON.parse(saved) as UserPrefs : defaultPrefs;
}

// almost certainly not needed, but it's here now
let locked = false;
export function saveUserPrefs(prefs: Partial<UserPrefs>): void {
  if (!localStorage) return;

  while (locked);

  locked = true;
  const currentPrefs = getUserPrefs();

  localStorage.setItem("prefs", JSON.stringify({ ...currentPrefs, ...prefs }));

  locked = false;
}
