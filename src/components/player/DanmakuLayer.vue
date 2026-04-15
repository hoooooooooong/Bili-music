<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import type { Danmaku } from "@/types";

const props = withDefaults(
  defineProps<{
    danmakuList: Danmaku[];
    currentTime: number;
    duration: number;
    playing: boolean;
  }>(),
  {
    danmakuList: () => [],
    currentTime: 0,
    duration: 0,
    playing: false,
  }
);

const containerRef = ref<HTMLElement | null>(null);
const trackCount = 12;
const trackHeight = 36;
const trackOccupiedUntil: Float64Array = new Float64Array(trackCount);
let emitIndex = 0;
let rafId: number | null = null;
const POOL_SIZE = 50;
const SPEED = 200; // pixels per second — faster scroll to reduce density
const MIN_TRACK_GAP = 0.8; // minimum seconds between danmaku on same track

const pool: HTMLDivElement[] = [];
let poolReady = false;

function initPool() {
  if (poolReady || !containerRef.value) return;
  for (let i = 0; i < POOL_SIZE; i++) {
    const el = document.createElement("div");
    el.className = "danmaku-item";
    el.style.display = "none";
    containerRef.value.appendChild(el);
    pool.push(el);
  }
  poolReady = true;
}

function resetTracks() {
  trackOccupiedUntil.fill(0);
}

function findTrack(now: number): number {
  let bestTrack = -1;
  let earliestEnd = Infinity;
  for (let i = 0; i < trackCount; i++) {
    if (trackOccupiedUntil[i] <= now && trackOccupiedUntil[i] < earliestEnd) {
      earliestEnd = trackOccupiedUntil[i];
      bestTrack = i;
    }
  }
  // If all tracks occupied, find the one that frees up soonest
  if (bestTrack === -1) {
    let soonest = Infinity;
    for (let i = 0; i < trackCount; i++) {
      if (trackOccupiedUntil[i] < soonest) {
        soonest = trackOccupiedUntil[i];
        bestTrack = i;
      }
    }
  }
  return bestTrack;
}

function acquireEl(): HTMLDivElement | null {
  for (const el of pool) {
    if (el.style.display === "none") return el;
  }
  return null;
}

function releaseEl(el: HTMLDivElement) {
  el.style.display = "none";
  el.style.animation = "";
  el.style.animationPlayState = "";
}

function launchDanmaku(d: Danmaku, now: number) {
  const el = acquireEl();
  if (!el) return;

  const track = findTrack(now);
  const containerWidth = containerRef.value?.clientWidth || 800;

  el.textContent = d.content;
  el.style.color = d.color;
  el.style.display = "block";
  el.style.top = track * trackHeight + 8 + "px";

  const textWidth = el.scrollWidth;
  const totalDist = containerWidth + textWidth;
  const duration = totalDist / SPEED;

  // Track is occupied until this danmaku has fully entered + gap
  trackOccupiedUntil[track] = now + (containerWidth + textWidth * 0.5) / SPEED + MIN_TRACK_GAP;

  el.style.setProperty("--scroll-from", `${containerWidth}px`);
  el.style.setProperty("--scroll-to", `-${textWidth}px`);

  el.style.animation = "none";
  el.offsetWidth; // force reflow
  el.style.animation = `danmaku-scroll ${duration}s linear forwards`;
  el.style.animationPlayState = props.playing ? "running" : "paused";

  el.addEventListener("animationend", () => releaseEl(el), { once: true });
}

// Dedup: skip danmaku too close in time to the last launched one
let lastLaunchTime = -1;

function tick() {
  if (!props.playing) {
    rafId = requestAnimationFrame(tick);
    return;
  }

  const now = props.currentTime;

  while (emitIndex < props.danmakuList.length && props.danmakuList[emitIndex].progress <= now) {
    const d = props.danmakuList[emitIndex];
    // Skip if too close to previous launch (reduce density)
    if (d.progress - lastLaunchTime >= 0.3) {
      launchDanmaku(d, now);
      lastLaunchTime = d.progress;
    }
    emitIndex++;
  }

  rafId = requestAnimationFrame(tick);
}

function clearAllActive() {
  for (const el of pool) {
    releaseEl(el);
  }
  resetTracks();
  lastLaunchTime = -1;
}

function resetEmitIndex() {
  const now = props.currentTime;
  let idx = 0;
  for (let i = 0; i < props.danmakuList.length; i++) {
    if (props.danmakuList[i].progress > now) {
      idx = i;
      break;
    }
    idx = i + 1;
  }
  emitIndex = idx;
  lastLaunchTime = now;
}

watch(
  () => props.danmakuList,
  () => {
    clearAllActive();
    emitIndex = 0;
  }
);

watch(
  () => props.playing,
  (playing) => {
    for (const el of pool) {
      if (el.style.display !== "none") {
        el.style.animationPlayState = playing ? "running" : "paused";
      }
    }
  }
);

// Detect seek: large jump in currentTime
let lastTime = 0;
watch(
  () => props.currentTime,
  (t) => {
    if (Math.abs(t - lastTime) > 2) {
      clearAllActive();
      resetEmitIndex();
    }
    lastTime = t;
  }
);

onMounted(() => {
  nextTick(() => {
    initPool();
    rafId = requestAnimationFrame(tick);
  });
});

onUnmounted(() => {
  if (rafId != null) cancelAnimationFrame(rafId);
  pool.forEach((el) => el.remove());
  pool.length = 0;
  poolReady = false;
});
</script>

<template>
  <div ref="containerRef" class="danmaku-layer"></div>
</template>

<style scoped>
.danmaku-layer {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: 5;
}
</style>

<style>
.danmaku-item {
  position: absolute;
  left: 0;
  white-space: nowrap;
  font-size: 15px;
  line-height: 20px;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8), 0 0 6px rgba(0, 0, 0, 0.6);
  will-change: transform;
  padding: 0 12px;
  font-weight: 500;
}

@keyframes danmaku-scroll {
  from {
    transform: translateX(var(--scroll-from));
  }
  to {
    transform: translateX(var(--scroll-to));
  }
}
</style>
