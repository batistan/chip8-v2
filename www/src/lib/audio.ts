import { createEffect } from "solid-js";
import { muted } from "../state";

let audioCtx: AudioContext | null = null;

export type CleanupAudioCallback = () => void;

export function setupAudio(): CleanupAudioCallback {
  if (!audioCtx) {
    audioCtx = new AudioContext();
  }

  const oscillator = new OscillatorNode(audioCtx);
  oscillator.start();

  const gain = new GainNode(audioCtx, { gain: 0 });

  oscillator.connect(gain).connect(audioCtx.destination);

  createEffect(() => {
    if (muted()) {
      gain.gain.setTargetAtTime(0, audioCtx!.currentTime, 0.005);
     } else {
      gain.gain.setTargetAtTime(0.1, audioCtx!.currentTime, 0.005);
     }
  })

  return () => {
  }
}