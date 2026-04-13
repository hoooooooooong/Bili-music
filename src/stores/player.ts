import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Song, PlayMode, LyricsData } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";

const audio = new Audio();

export const usePlayerStore = defineStore("player", () => {
  const currentSong = ref<Song | null>(null);
  const playlist = ref<Song[]>([]);
  const currentIndex = ref(-1);
  const playMode = ref<PlayMode>("sequential");
  const isPlaying = ref(false);
  const currentTime = ref(0);
  const duration = ref(0);
  const volume = ref(1);
  const lyrics = ref<LyricsData | null>(null);
  const loadingLyrics = ref(false);
  const audioUrl = ref("");
  const coverUrl = ref("");
  const buffering = ref(false);
  const playError = ref("");

  const progress = computed(() =>
    duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
  );

  function setVolume(val: number) {
    volume.value = Math.max(0, Math.min(1, val));
    audio.volume = volume.value;
  }

  async function playSong(song: Song, list?: Song[]) {
    currentSong.value = song;

    if (list) {
      playlist.value = list;
      currentIndex.value = list.findIndex((s) => s.bvid === song.bvid);
    } else if (
      currentIndex.value === -1 ||
      playlist.value[currentIndex.value]?.bvid !== song.bvid
    ) {
      playlist.value = [song];
      currentIndex.value = 0;
    }

    try {
      buffering.value = true;
      playError.value = "";
      coverUrl.value = song.coverUrl || `bili-cover://${song.bvid}`;

      const filePath = await invoke<string>("stream_audio", { bvid: song.bvid });
      const fileData = await readFile(filePath);
      const blob = new Blob([fileData], { type: "audio/mp4" });
      if (audioUrl.value) URL.revokeObjectURL(audioUrl.value);
      const blobUrl = URL.createObjectURL(blob);
      audioUrl.value = blobUrl;
      audio.src = blobUrl;
      audio.volume = volume.value;

      await audio.play();
      isPlaying.value = true;
    } catch (e: any) {
      playError.value = typeof e === "string" ? e : e.message || String(e);
      console.error("Failed to play:", e);
    } finally {
      buffering.value = false;
    }

    fetchLyrics(song.bvid);
  }

  function togglePlay() {
    if (!currentSong.value) return;
    if (isPlaying.value) {
      audio.pause();
      isPlaying.value = false;
    } else {
      audio.play().then(() => {
        isPlaying.value = true;
      }).catch(() => {});
    }
  }

  function seek(time: number) {
    audio.currentTime = time;
    currentTime.value = time;
  }

  function seekByPercent(percent: number) {
    if (duration.value > 0) {
      seek((percent / 100) * duration.value);
    }
  }

  async function next() {
    if (playlist.value.length === 0) return;
    if (playMode.value === "random") {
      const idx = Math.floor(Math.random() * playlist.value.length);
      currentIndex.value = idx;
      await playSong(playlist.value[idx]);
    } else {
      const nextIdx = (currentIndex.value + 1) % playlist.value.length;
      currentIndex.value = nextIdx;
      await playSong(playlist.value[nextIdx]);
    }
  }

  async function prev() {
    if (playlist.value.length === 0) return;
    if (audio.currentTime > 3) {
      seek(0);
      return;
    }
    if (playMode.value === "random") {
      const idx = Math.floor(Math.random() * playlist.value.length);
      currentIndex.value = idx;
      await playSong(playlist.value[idx]);
    } else {
      const prevIdx =
        currentIndex.value <= 0
          ? playlist.value.length - 1
          : currentIndex.value - 1;
      currentIndex.value = prevIdx;
      await playSong(playlist.value[prevIdx]);
    }
  }

  function togglePlayMode() {
    const modes: PlayMode[] = ["sequential", "loop", "random"];
    const idx = modes.indexOf(playMode.value);
    playMode.value = modes[(idx + 1) % modes.length];
  }

  function addToPlaylist(song: Song) {
    if (!playlist.value.some((s) => s.bvid === song.bvid)) {
      playlist.value.push(song);
    }
  }

  function removeFromPlaylist(index: number) {
    playlist.value.splice(index, 1);
    if (index < currentIndex.value) {
      currentIndex.value--;
    } else if (index === currentIndex.value) {
      if (playlist.value.length === 0) {
        currentIndex.value = -1;
        currentSong.value = null;
        audio.pause();
        audio.src = "";
        isPlaying.value = false;
      } else {
        currentIndex.value = Math.min(
          currentIndex.value,
          playlist.value.length - 1
        );
        playSong(playlist.value[currentIndex.value]);
      }
    }
  }

  function clearPlaylist() {
    playlist.value = [];
    currentIndex.value = -1;
    currentSong.value = null;
    audio.pause();
    audio.src = "";
    isPlaying.value = false;
  }

  async function fetchLyrics(bvid: string) {
    loadingLyrics.value = true;
    try {
      lyrics.value = await invoke<LyricsData>("fetch_lyrics", { bvid });
    } catch {
      lyrics.value = null;
    } finally {
      loadingLyrics.value = false;
    }
  }

  audio.addEventListener("timeupdate", () => {
    currentTime.value = audio.currentTime;
  });

  audio.addEventListener("loadedmetadata", () => {
    duration.value = audio.duration;
    console.log("[audio] loadedmetadata, duration:", audio.duration, "src:", audio.src);
  });

  audio.addEventListener("error", (e) => {
    const err = audio.error;
    console.error("[audio] error:", err?.code, err?.message, "src:", audio.src);
    playError.value = err ? `音频加载失败 (${err.code}): ${err.message}` : "音频加载失败";
  });

  audio.addEventListener("ended", async () => {
    if (playMode.value === "loop") {
      seek(0);
      audio.play();
    } else {
      await next();
    }
  });

  audio.addEventListener("pause", () => {
    isPlaying.value = false;
  });

  audio.addEventListener("play", () => {
    isPlaying.value = true;
  });

  return {
    currentSong,
    playlist,
    currentIndex,
    playMode,
    isPlaying,
    currentTime,
    duration,
    volume,
    progress,
    lyrics,
    loadingLyrics,
    audioUrl,
    coverUrl,
    buffering,
    playError,
    setVolume,
    playSong,
    togglePlay,
    seek,
    seekByPercent,
    next,
    prev,
    togglePlayMode,
    addToPlaylist,
    removeFromPlaylist,
    clearPlaylist,
    fetchLyrics,
  };
});
