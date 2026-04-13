<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { usePlayerStore } from "@/stores/player";

const player = usePlayerStore();
const canvas = ref<HTMLCanvasElement | null>(null);
let ctx: CanvasRenderingContext2D | null = null;
let animFrame: number | null = null;
let analyser: AnalyserNode | null = null;
let audioCtx: AudioContext | null = null;

function setup() {
  if (!canvas.value) return;
  ctx = canvas.value.getContext("2d");
  audioCtx = new AudioContext();

  const audioEl = document.querySelector("audio") as HTMLAudioElement;
  if (!audioEl) return;

  const source = audioCtx.createMediaElementSource(audioEl);
  analyser = audioCtx.createAnalyser();
  analyser.fftSize = 128;

  source.connect(analyser);
  analyser.connect(audioCtx.destination);

  draw();
}

function draw() {
  if (!ctx || !analyser || !canvas.value) return;
  const { width, height } = canvas.value;
  const bufferLength = analyser.frequencyBinCount;
  const dataArray = new Uint8Array(bufferLength);

  function render() {
    animFrame = requestAnimationFrame(render);
    analyser!.getByteFrequencyData(dataArray);
    ctx!.clearRect(0, 0, width, height);

    const barWidth = (width / bufferLength) * 1.5;
    let x = 0;

    for (let i = 0; i < bufferLength; i++) {
      const barHeight = (dataArray[i] / 255) * height * 0.8;
      const gradient = ctx!.createLinearGradient(
        0,
        height,
        0,
        height - barHeight
      );
      gradient.addColorStop(0, "rgba(251, 114, 153, 0.3)");
      gradient.addColorStop(1, "rgba(251, 114, 153, 0.8)");
      ctx!.fillStyle = gradient;
      ctx!.beginPath();
      ctx!.roundRect(x, height - barHeight, barWidth - 2, barHeight, 2);
      ctx!.fill();
      x += barWidth;
    }
  }

  render();
}

onMounted(() => setup());
onUnmounted(() => {
  if (animFrame) cancelAnimationFrame(animFrame);
  if (audioCtx) audioCtx.close();
});
</script>

<template>
  <canvas ref="canvas" class="audio-visualizer" width="240" height="60" />
</template>

<style scoped>
.audio-visualizer {
  display: block;
}
</style>
