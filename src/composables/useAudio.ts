import { ref, onUnmounted } from "vue";
import { usePlayerStore } from "@/stores/player";

export function useAudio() {
  const player = usePlayerStore();
  const analyser = ref<AnalyserNode | null>(null);
  const audioContext = ref<AudioContext | null>(null);

  function setupAudioContext() {
    if (audioContext.value) return;
    const ctx = new AudioContext();
    const audioEl = document.querySelector("audio") as HTMLAudioElement;
    if (!audioEl) return;

    const source = ctx.createMediaElementSource(audioEl);
    const node = ctx.createAnalyser();
    node.fftSize = 256;

    source.connect(node);
    node.connect(ctx.destination);

    audioContext.value = ctx;
    analyser.value = node;
  }

  function getFrequencyData(): Uint8Array {
    if (!analyser.value) return new Uint8Array(0);
    const data = new Uint8Array(analyser.value.frequencyBinCount);
    analyser.value.getByteFrequencyData(data);
    return data;
  }

  onUnmounted(() => {
    if (audioContext.value) {
      audioContext.value.close();
    }
  });

  return {
    setupAudioContext,
    getFrequencyData,
    analyser,
  };
}
