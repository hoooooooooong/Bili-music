import { watch, type MaybeRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { LyricsData, Song } from "@/types";
import { toValue } from "vue";

export function useCrossWindowSync(
  currentTime: MaybeRef<number>,
  lyrics: MaybeRef<LyricsData | null>,
  currentSong: MaybeRef<Song | null>,
  isPlaying: MaybeRef<boolean>,
  duration: MaybeRef<number>,
  coverUrl: MaybeRef<string>,
) {
  function syncState() {
    const lrc = toValue(lyrics);
    const state = {
      currentTime: toValue(currentTime),
      duration: toValue(duration),
      isPlaying: toValue(isPlaying),
      currentSong: toValue(currentSong) ?? null,
      coverUrl: toValue(coverUrl),
      lyrics: lrc ? lrc.lyrics : [],
    };
    invoke("update_player_state", { newState: state }).catch(() => {});
  }

  // Sync isPlaying immediately (no throttle) — pausing stops currentTime updates,
  // so a throttled watch would never fire again and the state would be stale.
  watch([isPlaying], () => {
    syncState();
  });

  // Throttle frequent updates from currentTime changes
  let _lastSyncTime = 0;
  watch(
    [currentTime, lyrics, currentSong, duration, coverUrl],
    () => {
      const now = Date.now();
      if (now - _lastSyncTime < 200 && _lastSyncTime > 0) return;
      _lastSyncTime = now;
      syncState();
    },
    { immediate: true },
  );
}
