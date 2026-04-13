import { defineStore } from "pinia";
import { ref } from "vue";

const MAX_OFFSET = 10;
const STEP = 0.5;

export const useLyricOffsetsStore = defineStore(
  "lyricOffsets",
  () => {
    const offsets = ref<Record<string, number>>({});
    const version = ref(0);

    function getOffset(bvid: string): number {
      return offsets.value[bvid] ?? 0;
    }

    function setOffset(bvid: string, offset: number) {
      const clamped = Math.max(-MAX_OFFSET, Math.min(MAX_OFFSET, offset));
      if (Math.abs(clamped) < 0.01) {
        clearOffset(bvid);
        return;
      }
      offsets.value[bvid] = Math.round(clamped / STEP) * STEP;
      version.value++;
    }

    function clearOffset(bvid: string) {
      delete offsets.value[bvid];
      version.value++;
    }

    return { offsets, version, getOffset, setOffset, clearOffset };
  },
  {
    persist: true,
  }
);
