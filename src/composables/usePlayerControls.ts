import { ref, computed } from "vue";
import { usePlayerStore } from "@/stores/player";

export function usePlayerControls() {
  const player = usePlayerStore();

  const showVolume = ref(false);
  const showSleepTimer = ref(false);

  const sleepTimerDisplay = computed(() => {
    const r = player.sleepTimerRemaining;
    const m = Math.floor(r / 60);
    const s = r % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  });

  const sleepTimerPresets = [10, 15, 30, 60, 90];

  function setSleepTimerAndClose(minutes: number) {
    player.setSleepTimer(minutes);
    showSleepTimer.value = false;
  }

  function clearSleepTimerAndClose() {
    player.clearSleepTimer();
    showSleepTimer.value = false;
  }

  function toggleMute() {
    player.setVolume(player.volume > 0 ? 0 : 0.7);
  }

  return {
    showVolume,
    showSleepTimer,
    sleepTimerDisplay,
    sleepTimerPresets,
    setSleepTimerAndClose,
    clearSleepTimerAndClose,
    toggleMute,
  };
}
