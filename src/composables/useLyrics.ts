import { ref, computed, watch } from "vue";
import { usePlayerStore } from "@/stores/player";
import { findCurrentLine } from "@/utils/lrc-parser";
import type { LyricLine } from "@/types";

export function useLyrics() {
  const player = usePlayerStore();
  const currentLineIndex = ref(-1);
  const userScrolled = ref(false);
  const scrollTimer = ref<ReturnType<typeof setTimeout> | null>(null);

  const lyrics = computed(() => player.lyrics?.lyrics || []);
  const hasLyrics = computed(() => lyrics.value.length > 0);

  watch(
    () => player.currentTime,
    (time) => {
      if (userScrolled.value) return;
      const idx = findCurrentLine(lyrics.value, time);
      if (idx !== currentLineIndex.value) {
        currentLineIndex.value = idx;
      }
    }
  );

  function onUserScroll() {
    userScrolled.value = true;
    if (scrollTimer.value) clearTimeout(scrollTimer.value);
    scrollTimer.value = setTimeout(() => {
      userScrolled.value = false;
    }, 3000);
  }

  function seekToLine(line: LyricLine) {
    player.seek(line.time);
    currentLineIndex.value = lyrics.value.indexOf(line);
  }

  return {
    currentLineIndex,
    lyrics,
    hasLyrics,
    onUserScroll,
    seekToLine,
  };
}
