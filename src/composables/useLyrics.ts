import { ref, computed, watch } from "vue";
import { usePlayerStore } from "@/stores/player";
import { useLyricOffsetsStore } from "@/stores/lyricOffsets";
import { findCurrentLine } from "@/utils/lrc-parser";
import type { LyricLine } from "@/types";

export function useLyrics() {
  const player = usePlayerStore();
  const lyricOffsets = useLyricOffsetsStore();
  const currentLineIndex = ref(-1);
  const userScrolled = ref(false);
  const scrollTimer = ref<ReturnType<typeof setTimeout> | null>(null);

  const lyrics = computed(() => player.lyrics?.lyrics || []);
  const hasLyrics = computed(() => lyrics.value.length > 0);

  const currentOffset = computed(() => {
    return player.currentSong
      ? lyricOffsets.getOffset(player.currentSong.bvid)
      : 0;
  });

  function getOffset(): number {
    return player.currentSong
      ? lyricOffsets.getOffset(player.currentSong.bvid)
      : 0;
  }

  let lastTime = -1;
  watch(
    () => player.currentTime,
    (time) => {
      if (userScrolled.value) return;
      // Throttle: skip if time changed < 200ms
      if (time - lastTime < 0.2 && lastTime >= 0) return;
      lastTime = time;
      const idx = findCurrentLine(lyrics.value, time - getOffset());
      if (idx !== currentLineIndex.value) {
        currentLineIndex.value = idx;
      }
    }
  );

  watch(() => lyricOffsets.version, () => {
    lastTime = -1;
    if (lyrics.value.length > 0) {
      const idx = findCurrentLine(
        lyrics.value,
        player.currentTime - getOffset()
      );
      if (idx !== currentLineIndex.value) {
        currentLineIndex.value = idx;
      }
    }
  });

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
    currentOffset,
    onUserScroll,
    seekToLine,
  };
}
