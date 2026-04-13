<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { usePlayerStore } from "@/stores/player";

const player = usePlayerStore();
const canvas = ref<HTMLCanvasElement | null>(null);
let ctx: CanvasRenderingContext2D | null = null;
let animFrame: number | null = null;
let w = 280;
let h = 72;

function render() {
  if (!ctx || !canvas.value) return;
  animFrame = requestAnimationFrame(render);

  if (!player.isPlaying || document.hidden) return;

  const analyser = player.getAnalyser();
  if (!analyser) return;

  const bufferLength = analyser.frequencyBinCount;
  const dataArray = new Uint8Array(bufferLength);
  analyser.getByteFrequencyData(dataArray);

  ctx.clearRect(0, 0, w, h);

  // 只取前一半频段（低频和中频更有表现力）
  const count = Math.floor(bufferLength * 0.6);
  const gap = 3;
  const barWidth = (w - gap * (count - 1)) / count;
  const maxBarH = h * 0.85;
  const radius = barWidth / 2;

  for (let i = 0; i < count; i++) {
    const val = dataArray[i] / 255;
    const barH = Math.max(3, val * maxBarH);
    const x = i * (barWidth + gap);

    const gradient = ctx.createLinearGradient(0, h, 0, h - barH);
    gradient.addColorStop(0, "rgba(251, 114, 153, 0.2)");
    gradient.addColorStop(0.5, "rgba(251, 114, 153, 0.6)");
    gradient.addColorStop(1, "rgba(252, 155, 180, 0.9)");
    ctx.fillStyle = gradient;

    ctx.beginPath();
    ctx.roundRect(x, h - barH, barWidth, barH, radius);
    ctx.fill();
  }
}

onMounted(() => {
  if (!canvas.value) return;
  const dpr = window.devicePixelRatio || 1;
  canvas.value.width = w * dpr;
  canvas.value.height = h * dpr;
  canvas.value.style.width = `${w}px`;
  canvas.value.style.height = `${h}px`;
  ctx = canvas.value.getContext("2d");
  ctx?.scale(dpr, dpr);
  render();
});

onUnmounted(() => {
  if (animFrame) cancelAnimationFrame(animFrame);
  ctx = null;
});
</script>

<template>
  <canvas ref="canvas" class="audio-visualizer" />
</template>

<style scoped>
.audio-visualizer {
  display: block;
}
</style>
