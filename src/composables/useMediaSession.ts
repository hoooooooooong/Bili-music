import type { Song } from "@/types";

/**
 * Web Media Session API — integrates with Windows SMTC via WebView2.
 * Zero npm dependencies, pure Web API.
 */
export function useMediaSession() {
  function init(
    onPlayPause: () => void,
    onNext: () => void,
    onPrev: () => void,
  ) {
    if (!("mediaSession" in navigator)) return;

    navigator.mediaSession.setActionHandler("play", () => {
      onPlayPause();
      navigator.mediaSession.playbackState = "playing";
    });

    navigator.mediaSession.setActionHandler("pause", () => {
      onPlayPause();
      navigator.mediaSession.playbackState = "paused";
    });

    navigator.mediaSession.setActionHandler("nexttrack", () => {
      onNext();
    });

    navigator.mediaSession.setActionHandler("previoustrack", () => {
      onPrev();
    });
  }

  function updateMetadata(song: Song | null) {
    if (!("mediaSession" in navigator)) return;

    if (!song) {
      navigator.mediaSession.metadata = null;
      return;
    }

    // Build artwork URL from coverUrl (may be bili-cover:// protocol or https://)
    const artworkUrl = song.coverUrl
      ? song.coverUrl.startsWith("http")
        ? song.coverUrl
        : window.location.origin + "/" + song.coverUrl
      : "";

    navigator.mediaSession.metadata = new MediaMetadata({
      title: song.title,
      artist: song.author,
      artwork: artworkUrl
        ? [{ src: artworkUrl, sizes: "256x256", type: "image/jpeg" }]
        : [],
    });
  }

  function updatePlaybackState(playing: boolean) {
    if (!("mediaSession" in navigator)) return;
    navigator.mediaSession.playbackState = playing ? "playing" : "paused";
  }

  return { init, updateMetadata, updatePlaybackState };
}
