<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { usePlayerStore } from "@/stores/player";
import type { VisualizerStyle } from "@/types";

const player = usePlayerStore();
const canvas = ref<HTMLCanvasElement | null>(null);
let ctx: CanvasRenderingContext2D | null = null;
let animFrame: number | null = null;

const SIZES: Record<VisualizerStyle, { w: number; h: number }> = {
  bars: { w: 280, h: 72 },
  wave: { w: 280, h: 72 },
  circle: { w: 200, h: 200 },
  dots: { w: 280, h: 72 },
  mirror: { w: 280, h: 72 },
};

let currentW = SIZES[player.visualizerStyle].w;
let currentH = SIZES[player.visualizerStyle].h;

function setupCanvas(w: number, h: number) {
  if (!canvas.value) return;
  const dpr = window.devicePixelRatio || 1;
  canvas.value.width = w * dpr;
  canvas.value.height = h * dpr;
  canvas.value.style.width = `${w}px`;
  canvas.value.style.height = `${h}px`;
  ctx = canvas.value.getContext("2d");
  ctx?.scale(dpr, dpr);
  currentW = w;
  currentH = h;
}

function drawBars(dataArray: Uint8Array, bufferLength: number) {
  if (!ctx) return;
  const w = currentW;
  const h = currentH;
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

function drawWave(dataArray: Uint8Array, bufferLength: number) {
  if (!ctx) return;
  const w = currentW;
  const h = currentH;
  const count = Math.floor(bufferLength * 0.6);
  const step = w / (count - 1);

  const gradient = ctx.createLinearGradient(0, 0, w, 0);
  gradient.addColorStop(0, "rgba(251, 114, 153, 0.9)");
  gradient.addColorStop(0.5, "rgba(252, 155, 180, 1)");
  gradient.addColorStop(1, "rgba(251, 114, 153, 0.9)");

  ctx.save();
  ctx.strokeStyle = gradient;
  ctx.lineWidth = 2;
  ctx.shadowColor = "rgba(251, 114, 153, 0.6)";
  ctx.shadowBlur = 8;

  const points: { x: number; y: number }[] = [];
  for (let i = 0; i < count; i++) {
    const val = dataArray[i] / 255;
    const x = i * step;
    const y = h - val * h * 0.85 - h * 0.075;
    points.push({ x, y });
  }

  ctx.beginPath();
  ctx.moveTo(points[0].x, points[0].y);
  for (let i = 1; i < points.length - 1; i++) {
    const xc = (points[i].x + points[i + 1].x) / 2;
    const yc = (points[i].y + points[i + 1].y) / 2;
    ctx.quadraticCurveTo(points[i].x, points[i].y, xc, yc);
  }
  const last = points[points.length - 1];
  ctx.lineTo(last.x, last.y);
  ctx.stroke();
  ctx.restore();
}

function drawCircle(dataArray: Uint8Array, bufferLength: number) {
  if (!ctx) return;
  const w = currentW;
  const h = currentH;
  const cx = w / 2;
  const cy = h / 2;
  const innerRadius = 35;
  const barCount = Math.floor(bufferLength / 2);

  for (let i = 0; i < barCount; i++) {
    const val = dataArray[i] / 255;
    const barLen = Math.max(2, val * 45);
    const angle = (i / barCount) * Math.PI * 2 - Math.PI / 2;
    const x1 = cx + Math.cos(angle) * innerRadius;
    const y1 = cy + Math.sin(angle) * innerRadius;
    const x2 = cx + Math.cos(angle) * (innerRadius + barLen);
    const y2 = cy + Math.sin(angle) * (innerRadius + barLen);

    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.strokeStyle = `rgba(251, 114, 153, ${0.3 + val * 0.7})`;
    ctx.lineWidth = 2.5;
    ctx.lineCap = "round";
    ctx.stroke();
  }
}

function drawDots(dataArray: Uint8Array, bufferLength: number) {
  if (!ctx) return;
  const w = currentW;
  const h = currentH;
  const count = Math.floor(bufferLength * 0.6);
  const cols = count;
  const rows = 3;
  const gapX = w / cols;
  const gapY = h / rows;

  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const val = dataArray[c] / 255;
      const rowFactor = 1 - r * 0.25;
      const radius = Math.max(1, val * rowFactor * 5);
      const alpha = 0.15 + val * rowFactor * 0.85;
      const x = gapX * c + gapX / 2;
      const y = gapY * r + gapY / 2;

      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(251, 114, 153, ${alpha})`;
      ctx.fill();
    }
  }
}

function drawMirror(dataArray: Uint8Array, bufferLength: number) {
  if (!ctx) return;
  const w = currentW;
  const h = currentH;
  const midY = h / 2;
  const count = Math.floor(bufferLength * 0.6);
  const gap = 3;
  const barWidth = (w - gap * (count - 1)) / count;
  const maxBarH = midY * 0.85;
  const radius = barWidth / 2;

  for (let i = 0; i < count; i++) {
    const val = dataArray[i] / 255;
    const barH = Math.max(2, val * maxBarH);
    const x = i * (barWidth + gap);

    // Upper bar
    const gradUp = ctx.createLinearGradient(0, midY, 0, midY - barH);
    gradUp.addColorStop(0, "rgba(251, 114, 153, 0.2)");
    gradUp.addColorStop(1, "rgba(252, 155, 180, 0.9)");
    ctx.fillStyle = gradUp;
    ctx.beginPath();
    ctx.roundRect(x, midY - barH, barWidth, barH, [radius, radius, 0, 0]);
    ctx.fill();

    // Lower bar (mirrored)
    const gradDown = ctx.createLinearGradient(0, midY, 0, midY + barH);
    gradDown.addColorStop(0, "rgba(251, 114, 153, 0.2)");
    gradDown.addColorStop(1, "rgba(252, 155, 180, 0.9)");
    ctx.fillStyle = gradDown;
    ctx.beginPath();
    ctx.roundRect(x, midY, barWidth, barH, [0, 0, radius, radius]);
    ctx.fill();
  }
}

const drawFns: Record<VisualizerStyle, (data: Uint8Array, len: number) => void> = {
  bars: drawBars,
  wave: drawWave,
  circle: drawCircle,
  dots: drawDots,
  mirror: drawMirror,
};

function render() {
  if (!ctx || !canvas.value) return;
  animFrame = requestAnimationFrame(render);

  if (!player.isPlaying || document.hidden) return;

  const analyser = player.getAnalyser();
  if (!analyser) return;

  const bufferLength = analyser.frequencyBinCount;
  const dataArray = new Uint8Array(bufferLength);
  analyser.getByteFrequencyData(dataArray);

  ctx.clearRect(0, 0, currentW, currentH);
  drawFns[player.visualizerStyle](dataArray, bufferLength);
}

watch(
  () => player.visualizerStyle,
  (style) => {
    const { w, h } = SIZES[style];
    if (w !== currentW || h !== currentH) {
      setupCanvas(w, h);
    }
  }
);

onMounted(() => {
  setupCanvas(SIZES[player.visualizerStyle].w, SIZES[player.visualizerStyle].h);
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
