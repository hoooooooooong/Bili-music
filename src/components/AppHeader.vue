<script setup lang="ts">
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NIcon } from "naive-ui";
import SearchBar from "./search/SearchBar.vue";
import { HomeOutline, DownloadOutline } from "@vicons/ionicons5";

const router = useRouter();
const route = useRoute();

const activeTab = computed(() => {
  if (route.path === "/downloads") return "downloads";
  return "home";
});

function switchTab(tab: string) {
  if (tab === "home") router.push("/");
  else if (tab === "downloads") router.push("/downloads");
}
</script>

<template>
  <div class="app-header">
    <div class="header-left">
      <button
        class="nav-btn"
        :class="{ active: activeTab === 'home' }"
        @click="switchTab('home')"
      >
        <NIcon size="18"><HomeOutline /></NIcon>
        <span>发现</span>
      </button>
      <button
        class="nav-btn"
        :class="{ active: activeTab === 'downloads' }"
        @click="switchTab('downloads')"
      >
        <NIcon size="18"><DownloadOutline /></NIcon>
        <span>下载</span>
      </button>
    </div>
    <SearchBar />
  </div>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  gap: 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--header-bg);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.nav-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.nav-btn.active {
  background: var(--accent-light);
  color: var(--accent-color);
  font-weight: 500;
}
</style>
