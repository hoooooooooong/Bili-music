import { onMounted, onUnmounted } from "vue";
import { usePlayerStore } from "@/stores/player";

export function useKeyboardShortcuts() {
  const player = usePlayerStore();

  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") return;

    switch (e.code) {
      case "Space":
        e.preventDefault();
        player.togglePlay();
        break;
      case "ArrowLeft":
        e.preventDefault();
        player.seek(Math.max(0, player.currentTime - 5));
        break;
      case "ArrowRight":
        e.preventDefault();
        player.seek(Math.min(player.duration, player.currentTime + 5));
        break;
      case "ArrowUp":
        e.preventDefault();
        player.setVolume(player.volume + 0.1);
        break;
      case "ArrowDown":
        e.preventDefault();
        player.setVolume(player.volume - 0.1);
        break;
      case "KeyM":
        player.setVolume(player.volume > 0 ? 0 : 0.7);
        break;
      case "KeyN":
        player.next();
        break;
      case "KeyP":
        player.prev();
        break;
      case "KeyL":
        player.togglePlayMode();
        break;
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
}
