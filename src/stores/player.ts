import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Song, PlayMode, LyricsData, VisualizerStyle } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { emit } from "@/utils/emitter";

const audio = new Audio();

// AudioContext + Analyser: 全局只初始化一次，供 AudioVisualizer 使用
let _audioCtx: AudioContext | null = null;
let _analyser: AnalyserNode | null = null;
function cleanupAudioContext() {
  if (_analyser) {
    _analyser.disconnect();
    _analyser = null;
  }
  if (_audioCtx) {
    _audioCtx.close().catch(() => {});
    _audioCtx = null;
  }
}

function getAnalyser(): AnalyserNode | null {
  if (_analyser) {
    if (_audioCtx?.state === "suspended") _audioCtx.resume();
    return _analyser;
  }
  try {
    _audioCtx = new AudioContext();
    const source = _audioCtx.createMediaElementSource(audio);
    _analyser = _audioCtx.createAnalyser();
    _analyser.fftSize = 128;
    source.connect(_analyser);
    _analyser.connect(_audioCtx.destination);
  } catch (e) {
    console.error("[player] getAnalyser failed:", e);
  }
  return _analyser;
}

export const usePlayerStore = defineStore("player", () => {
  const currentSong = ref<Song | null>(null);
  const playlist = ref<Song[]>([]);
  const currentIndex = ref(-1);
  const playMode = ref<PlayMode>("sequential");
  const isPlaying = ref(false);
  // Persisted play state: saved before audio.pause() resets isPlaying
  const _wasPlayingBeforeClose = ref(false);
  const currentTime = ref(0);
  const duration = ref(0);
  const volume = ref(1);
  const lyrics = ref<LyricsData | null>(null);
  const loadingLyrics = ref(false);
  const audioUrl = ref("");
  const coverUrl = ref("");
  const buffering = ref(false);
  const playError = ref("");
  const sleepTimerRemaining = ref(0);
  const sleepTimerTotal = ref(0);
  const visualizerStyle = ref<VisualizerStyle>("bars");
  let _sleepTimerInterval: ReturnType<typeof setInterval> | null = null;

  // Persistence: throttled time save (every 3s) + pending seek for restore
  const _savedCurrentTime = ref(0);
  let _timeSaveTimer: ReturnType<typeof setInterval> | null = null;
  const _pendingSeekTime = ref(0);

  function ensureTimeSaveTimer() {
    if (_timeSaveTimer) return;
    _timeSaveTimer = setInterval(() => {
      _savedCurrentTime.value = audio.currentTime;
    }, 3000);
  }

  const progress = computed(() =>
    duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
  );

  function setVolume(val: number) {
    volume.value = Math.max(0, Math.min(1, val));
    audio.volume = volume.value;
  }

  async function playSong(song: Song, list?: Song[], autoPlay: boolean = true, seekTime: number = 0) {
    _pendingSeekTime.value = seekTime;
    currentSong.value = song;
    emit("song:played", song);

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

      ensureTimeSaveTimer();

      if (autoPlay) {
        await audio.play();
        isPlaying.value = true;
        // 确保 AudioContext 在用户交互后恢复
        getAnalyser();
      }
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
      if (_pendingSeekTime.value > 0) {
        audio.currentTime = _pendingSeekTime.value;
        currentTime.value = _pendingSeekTime.value;
        _pendingSeekTime.value = 0;
      }
      audio.play().then(() => {
        isPlaying.value = true;
      }).catch(() => {});
    }
  }

  function pause() {
    audio.pause();
  }

  function setSleepTimer(minutes: number) {
    if (_sleepTimerInterval) clearInterval(_sleepTimerInterval);
    sleepTimerTotal.value = minutes * 60;
    sleepTimerRemaining.value = minutes * 60;
    _sleepTimerInterval = setInterval(() => {
      sleepTimerRemaining.value--;
      if (sleepTimerRemaining.value <= 0) {
        audio.pause();
        clearInterval(_sleepTimerInterval!);
        _sleepTimerInterval = null;
        sleepTimerRemaining.value = 0;
        sleepTimerTotal.value = 0;
      }
    }, 1000);
  }

  function clearSleepTimer() {
    if (_sleepTimerInterval) clearInterval(_sleepTimerInterval);
    _sleepTimerInterval = null;
    sleepTimerRemaining.value = 0;
    sleepTimerTotal.value = 0;
  }

  function setVisualizerStyle(style: VisualizerStyle) {
    visualizerStyle.value = style;
  }

  function seek(time: number) {
    audio.currentTime = time;
    currentTime.value = time;
  }

  let _seekTimer: ReturnType<typeof setTimeout> | null = null;

  function seekByPercent(percent: number) {
    if (duration.value <= 0) return;
    const time = (percent / 100) * duration.value;
    // Update visual position immediately
    currentTime.value = time;
    // Debounce the actual audio seek
    if (_seekTimer) clearTimeout(_seekTimer);
    _seekTimer = setTimeout(() => {
      audio.currentTime = time;
      _seekTimer = null;
    }, 80);
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

  function movePlaylistItem(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const [item] = playlist.value.splice(fromIndex, 1);
    playlist.value.splice(toIndex, 0, item);

    // Adjust currentIndex to follow the moved song
    if (fromIndex === currentIndex.value) {
      currentIndex.value = toIndex;
    } else if (fromIndex < currentIndex.value && toIndex >= currentIndex.value) {
      currentIndex.value--;
    } else if (fromIndex > currentIndex.value && toIndex <= currentIndex.value) {
      currentIndex.value++;
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
    if (_pendingSeekTime.value > 0) {
      audio.currentTime = _pendingSeekTime.value;
      currentTime.value = _pendingSeekTime.value;
    }
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

  async function restoreLastState() {
    const song = currentSong.value;
    const savedTime = _savedCurrentTime.value;
    const wasPlaying = _wasPlayingBeforeClose.value;
    if (!song) return;

    // Clear saved state to prevent stale restore on next restart
    _savedCurrentTime.value = 0;
    _wasPlayingBeforeClose.value = false;

    const list = playlist.value.length > 0 ? playlist.value : undefined;
    await playSong(song, list, wasPlaying, savedTime);
  }

  function cleanup() {
    if (_seekTimer) {
      clearTimeout(_seekTimer);
      _seekTimer = null;
    }
    // Save current playback position for next session
    _savedCurrentTime.value = audio.currentTime;
    _wasPlayingBeforeClose.value = isPlaying.value;
    if (_timeSaveTimer) {
      clearInterval(_timeSaveTimer);
      _timeSaveTimer = null;
    }
    audio.pause();
    audio.removeAttribute("src");
    if (audioUrl.value) {
      URL.revokeObjectURL(audioUrl.value);
      audioUrl.value = "";
    }
    cleanupAudioContext();
  }

  return {
    audio,
    getAnalyser,
    cleanup,
    restoreLastState,
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
    sleepTimerRemaining,
    sleepTimerTotal,
    visualizerStyle,
    setVisualizerStyle,
    setVolume,
    playSong,
    togglePlay,
    pause,
    setSleepTimer,
    clearSleepTimer,
    seek,
    seekByPercent,
    next,
    prev,
    togglePlayMode,
    addToPlaylist,
    movePlaylistItem,
    removeFromPlaylist,
    clearPlaylist,
    fetchLyrics,
  };
}, {
  persist: {
    pick: ['currentSong', 'playlist', 'currentIndex', 'playMode', '_savedCurrentTime', '_wasPlayingBeforeClose', 'visualizerStyle'],
  } as any,
});
