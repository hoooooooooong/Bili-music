import { ref, computed, watch } from "vue";
import { usePlayerStore } from "@/stores/player";
import { useLyricOffsetsStore } from "@/stores/lyricOffsets";
import { findCurrentLine } from "@/utils/lrc-parser";
import type { LyricLine } from "@/types";

// ── 模块级单例状态：所有组件共享 ──
const sharedCurrentLineIndex = ref(-1);
let sharedLastTime = -1;
let _watcherSetup = false;

function ensureWatcher() {
  if (_watcherSetup) return;
  _watcherSetup = true;

  const player = usePlayerStore();
  const lyricOffsets = useLyricOffsetsStore();

  watch(
    () => player.currentTime,
    (time) => {
      if (time - sharedLastTime < 0.2 && sharedLastTime >= 0) return;
      sharedLastTime = time;
      const lyrics = player.lyrics?.lyrics || [];
      const offset = player.currentSong
        ? lyricOffsets.getOffset(player.currentSong.bvid)
        : 0;
      const idx = findCurrentLine(lyrics, time - offset);
      if (idx !== sharedCurrentLineIndex.value) {
        sharedCurrentLineIndex.value = idx;
      }
    }
  );

  watch(() => lyricOffsets.version, () => {
    sharedLastTime = -1;
    const player = usePlayerStore();
    const lyricOffsets = useLyricOffsetsStore();
    const lyrics = player.lyrics?.lyrics || [];
    if (lyrics.length > 0) {
      const offset = player.currentSong
        ? lyricOffsets.getOffset(player.currentSong.bvid)
        : 0;
      const idx = findCurrentLine(lyrics, player.currentTime - offset);
      if (idx !== sharedCurrentLineIndex.value) {
        sharedCurrentLineIndex.value = idx;
      }
    }
  });

  // 切歌时重置
  watch(() => player.currentSong?.bvid, () => {
    sharedCurrentLineIndex.value = -1;
    sharedLastTime = -1;
  });

  // 歌词数据变化时重新计算当前行（歌词加载完成/导入/清除时触发）
  watch(
    () => player.lyrics,
    () => {
      sharedLastTime = -1;
      const lyrics = player.lyrics?.lyrics || [];
      if (lyrics.length > 0) {
        const offset = player.currentSong
          ? lyricOffsets.getOffset(player.currentSong.bvid)
          : 0;
        const idx = findCurrentLine(lyrics, player.currentTime - offset);
        if (idx !== sharedCurrentLineIndex.value) {
          sharedCurrentLineIndex.value = idx;
        }
      } else {
        sharedCurrentLineIndex.value = -1;
      }
    }
  );
}

export function useLyrics() {
  ensureWatcher();

  const player = usePlayerStore();
  const lyricOffsets = useLyricOffsetsStore();

  const lyrics = computed(() => player.lyrics?.lyrics || []);
  const hasLyrics = computed(() => lyrics.value.length > 0);

  const currentOffset = computed(() => {
    return player.currentSong
      ? lyricOffsets.getOffset(player.currentSong.bvid)
      : 0;
  });

  // 仅用于组件级的用户滚动状态（ScrollingLyrics 内部自行处理滚动暂停）
  const userScrolled = ref(false);
  const scrollTimer = ref<ReturnType<typeof setTimeout> | null>(null);

  function onUserScroll() {
    userScrolled.value = true;
    if (scrollTimer.value) clearTimeout(scrollTimer.value);
    scrollTimer.value = setTimeout(() => {
      userScrolled.value = false;
    }, 3000);
  }

  function seekToLine(line: LyricLine) {
    player.seek(line.time);
    sharedCurrentLineIndex.value = lyrics.value.indexOf(line);
  }

  return {
    currentLineIndex: sharedCurrentLineIndex,
    lyrics,
    hasLyrics,
    currentOffset,
    onUserScroll,
    seekToLine,
  };
}
