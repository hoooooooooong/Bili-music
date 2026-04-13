<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import type { LyricLine } from "@/types";

const props = defineProps<{
  lyrics: LyricLine[];
  currentIndex: number;
}>();

const emit = defineEmits<{
  scroll: [];
  seek: [line: LyricLine];
}>();

const container = ref<HTMLElement | null>(null);
const userScrolling = ref(false);
let scrollTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.currentIndex,
  async (idx) => {
    if (userScrolling.value || !container.value) return;
    if (idx < 0) return;

    await nextTick();
    const activeLine = container.value.children[idx] as
      | HTMLElement
      | undefined;
    if (!activeLine) return;

    const containerRect = container.value.getBoundingClientRect();
    const lineRect = activeLine.getBoundingClientRect();
    const offset =
      lineRect.top -
      containerRect.top -
      containerRect.height / 2 +
      lineRect.height / 2;

    container.value.scrollBy({ top: offset, behavior: "smooth" });
  }
);

function onScroll() {
  userScrolling.value = true;
  emit("scroll");
  if (scrollTimer) clearTimeout(scrollTimer);
  scrollTimer = setTimeout(() => {
    userScrolling.value = false;
  }, 3000);
}

function onClickLine(line: LyricLine) {
  emit("seek", line);
}
</script>

<template>
  <div ref="container" class="scrolling-lyrics" @scroll="onScroll">
    <div v-if="lyrics.length === 0" class="no-lyrics">
      <p>暂无歌词</p>
    </div>
    <div
      v-for="(line, index) in lyrics"
      :key="index"
      class="lyric-line"
      :class="{ active: index === currentIndex }"
      @click="onClickLine(line)"
    >
      {{ line.text }}
    </div>
  </div>
</template>

<style scoped>
.scrolling-lyrics {
  height: 100%;
  overflow-y: auto;
  padding: 20px 0;
  mask-image: linear-gradient(
    to bottom,
    transparent 0%,
    black 15%,
    black 85%,
    transparent 100%
  );
}

.no-lyrics {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: rgba(255, 255, 255, 0.3);
  font-size: 16px;
}

.lyric-line {
  padding: 8px 16px;
  font-size: 15px;
  color: rgba(255, 255, 255, 0.3);
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: center;
  line-height: 1.6;
}

.lyric-line:hover {
  color: rgba(255, 255, 255, 0.6);
}

.lyric-line.active {
  color: white;
  font-size: 17px;
  font-weight: 500;
  transform: scale(1.02);
}

.scrolling-lyrics::-webkit-scrollbar {
  width: 4px;
}

.scrolling-lyrics::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}
</style>
