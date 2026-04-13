<script setup lang="ts">
import { NIcon } from "naive-ui";
import { useSettingsStore, setThemeClickOrigin } from "@/stores/settings";
import { SettingsOutline, SunnyOutline, MoonOutline } from "@vicons/ionicons5";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";

const settingsStore = useSettingsStore();
const router = useRouter();

function toggleTheme(e: MouseEvent) {
  setThemeClickOrigin(e.clientX, e.clientY);
  const next = settingsStore.theme === "dark" ? "light" : "dark";
  settingsStore.setTheme(next);
}

function minimize() {
  getCurrentWindow().minimize();
}

function maximize() {
  getCurrentWindow().toggleMaximize();
}

function close() {
  getCurrentWindow().close();
}

function onDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement).closest("button")) return;
  getCurrentWindow().startDragging();
}

function onDoubleClick() {
  getCurrentWindow().toggleMaximize();
}
</script>

<template>
  <div class="titlebar" @mousedown="onDragStart" @dblclick="onDoubleClick">
    <div class="titlebar-left">
      <span class="app-logo">&#9835; Bili Music</span>
    </div>
    <div class="titlebar-right">
      <button class="tb-btn" @click="toggleTheme" title="切换主题">
        <NIcon size="16">
          <SunnyOutline v-if="settingsStore.theme === 'dark'" />
          <MoonOutline v-else />
        </NIcon>
      </button>
      <button class="tb-btn" @click="router.push('/settings')" title="设置">
        <NIcon size="16"><SettingsOutline /></NIcon>
      </button>
      <div class="window-controls">
        <button class="tb-btn wc-btn" @click="minimize" title="最小化">
          <span class="wc-icon minimize"></span>
        </button>
        <button class="tb-btn wc-btn" @click="maximize" title="最大化">
          <span class="wc-icon maximize"></span>
        </button>
        <button class="tb-btn wc-btn wc-close" @click="close" title="关闭">
          <span class="wc-icon close"></span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 8px 0 16px;
  background: var(--header-bg);
  border-bottom: 1px solid var(--border-color);
  user-select: none;
  flex-shrink: 0;
  app-region: drag;
  -webkit-app-region: drag;
}

.titlebar-left {
  display: flex;
  align-items: center;
}

.app-logo {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-color);
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 2px;
  app-region: no-drag;
  -webkit-app-region: no-drag;
}

.tb-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--app-text);
  transition: background 0.15s;
  app-region: no-drag;
  -webkit-app-region: no-drag;
}

.tb-btn:hover {
  background: var(--card-hover);
}

.window-controls {
  display: flex;
  margin-left: 4px;
}

.wc-btn {
  width: 46px;
  border-radius: 0;
}

.wc-close:hover {
  background: #e81123;
  color: white;
}

.wc-icon {
  display: block;
}

.wc-icon.minimize {
  width: 10px;
  height: 1px;
  background: currentColor;
  margin-top: 6px;
}

.wc-icon.maximize {
  width: 9px;
  height: 9px;
  border: 1.5px solid currentColor;
  border-radius: 1px;
}

.wc-icon.close {
  position: relative;
  width: 12px;
  height: 12px;
}
.wc-icon.close::before,
.wc-icon.close::after {
  content: "";
  position: absolute;
  left: 50%;
  top: 50%;
  width: 10px;
  height: 1.5px;
  background: currentColor;
}
.wc-icon.close::before {
  transform: translate(-50%, -50%) rotate(45deg);
}
.wc-icon.close::after {
  transform: translate(-50%, -50%) rotate(-45deg);
}
</style>
