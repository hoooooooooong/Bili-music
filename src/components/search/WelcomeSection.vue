<script setup lang="ts">
import { NIcon } from "naive-ui";
import { MusicalNotesOutline } from "@vicons/ionicons5";

const hotKeywords = [
  "周杰伦",
  "Aimer",
  "YOASOBI",
  "LiSA",
  "米津玄師",
  "Ado",
  "EVA",
  "原神",
];

const emit = defineEmits<{ search: [keyword: string] }>();
</script>

<template>
  <div class="welcome-section">
    <div class="welcome-icon">
      <NIcon size="48" color="var(--accent-color)">
        <MusicalNotesOutline />
      </NIcon>
    </div>
    <h2 class="welcome-title">搜索 B 站音乐</h2>
    <p class="welcome-desc">搜索你喜欢的音乐，在线播放或下载为 MP3</p>
    <div class="hot-keywords">
      <button
        v-for="kw in hotKeywords"
        :key="kw"
        class="keyword-tag"
        @click="emit('search', kw)"
      >
        {{ kw }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.welcome-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.welcome-icon {
  margin-bottom: 16px;
}

.welcome-title {
  font-size: 22px;
  font-weight: 600;
  color: var(--app-text);
  margin-bottom: 8px;
}

.welcome-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.hot-keywords {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
  max-width: 400px;
}

.keyword-tag {
  padding: 6px 16px;
  border-radius: 16px;
  font-size: 13px;
  color: var(--app-text);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  transition: all 0.15s;
}

.keyword-tag:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}
</style>
