import { createSignal } from "solid-js";
import type { Internals } from "./lib/internals";

export type AppError = {
  id: string;
  message: string;
};

const [errors, setErrors] = createSignal<readonly AppError[]>([]);

function randomId(): string {
  // sure, not as good as a uuid
  // but how many errors are we likely to have at any given time anyway
  return Math.random() // 0.21730609204670115
    .toString(36) // "0.7tmmseo7hsd"
    .substring(2, 8); // "7tmmse"
}

export function addError(message: string): void {
  setErrors((p) => [...p, { id: randomId(), message }]);
}

export function getErrors(): readonly AppError[] {
  return errors();
}

export function dismissError(id: string): void {
  setErrors((prev) => prev.filter((e) => e.id !== id));
}

export function clearErrors(): void {
  setErrors([]);
}

export type EmulationState = "stopped" | "running" | "paused" | "error";
export const [emulationState, setEmulationState] =
  createSignal<EmulationState>("stopped");

export type CurrentRom = { name: string; bytes: Uint8Array } | null;
export const [currentRom, setCurrentRom] = createSignal<CurrentRom>(null);

export const SPEEDS = [0.25, 0.5, 1, 2, 4] as const;
export type Speed = (typeof SPEEDS)[number];
export const [speed, setSpeed] = createSignal<Speed>(1);

export const [internals, setInternals] = createSignal<Internals | null>(null);

export function togglePause(): void {
  setEmulationState((s) =>
    s === "running" ? "paused" : s === "paused" ? "running" : s,
  );
}

export const [fps, setFps] = createSignal<number>(0);

export const [scale, setScale] = createSignal<number>(10);

export const [pressedKeys, setPressedKeys] = createSignal<ReadonlySet<number>>(
  new Set(),
);

export const [muted, setMuted] = createSignal<boolean>(false);
