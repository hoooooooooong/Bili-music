<script setup lang="ts">
import { usePlayerStore } from "@/stores/player";

const player = usePlayerStore();
</script>

<template>
  <div class="spinning-disc" :class="{ spinning: player.isPlaying }">
    <div class="disc-outer">
      <div class="disc-inner">
        <img
          :src="
            player.currentSong
              ? (player.currentSong.coverUrl || `bili-cover://${player.currentSong.bvid}`)
              : ''
          "
          :alt="player.currentSong?.title"
          class="disc-cover"
        />
        <div class="disc-center"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.spinning-disc {
  width: 240px;
  height: 240px;
}

.disc-outer {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    #1a1a1a 0%,
    #1a1a1a 30%,
    #2a2a2a 30.5%,
    #1a1a1a 31%,
    #2a2a2a 50%,
    #1a1a1a 50.5%,
    #2a2a2a 70%,
    #1a1a1a 70.5%,
    #2a2a2a 90%,
    #1a1a1a 100%
  );
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
}

.disc-inner {
  width: 160px;
  height: 160px;
  border-radius: 50%;
  overflow: hidden;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.disc-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.disc-center {
  position: absolute;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #333;
  border: 3px solid #555;
}

.spinning .disc-outer {
  animation: spin 8s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
