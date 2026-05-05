// Keyboard API isn't supported in many browsers, so we need to declare its types ourselves, as they aren't defined by default
interface KeyboardLayoutMap extends ReadonlyMap<string, string> {}

interface Keyboard {
  getLayoutMap(): Promise<KeyboardLayoutMap>;
}

interface Navigator {
  readonly keyboard?: Keyboard;
}
