<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
  NIcon,
  NButton,
  NInput,
  NTag,
  NSwitch,
  useMessage,
} from "naive-ui";
import { EyeOutline, EyeOffOutline } from "@vicons/ionicons5";
import {
  ArrowBackOutline,
  FolderOpenOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  RefreshOutline,
  TrashOutline,
} from "@vicons/ionicons5";
import { useSettingsStore, setThemeClickOrigin } from "@/stores/settings";
import { PRESET_COLORS } from "@/utils/colorUtils";
import { useHistoryStore } from "@/stores/history";
import { audioCache, MAX_CACHE_SIZE } from "@/composables/useAudioCache";
import type { AudioFormat, AudioQuality } from "@/types";
import { useRouter } from "vue-router";

const router = useRouter();
const settingsStore = useSettingsStore();
const historyStore = useHistoryStore();
const message = useMessage();

const ffmpegOk = ref(false);
const showSessdata = ref(false);

async function setSessdata(val: string) {
  settingsStore.sessdata = val;
  await settingsStore.saveSettings();
  if (val) {
    message.success("SESSDATA 已保存，评论等功能将使用登录状态");
  }
}

// 数据统计
const totalPlays = computed(
  () => historyStore.history.reduce((sum, e) => sum + e.playCount, 0)
);
const uniqueSongs = computed(() => historyStore.history.length);
const topSongs = computed(() =>
  [...historyStore.history]
    .sort((a, b) => b.playCount - a.playCount)
    .slice(0, 10)
);

// 存储管理
const cacheSize = ref(0);
const cacheEntryCount = ref(0);
const cachePercent = computed(() =>
  MAX_CACHE_SIZE > 0 ? (cacheSize.value / MAX_CACHE_SIZE) * 100 : 0
);

function formatMB(bytes: number): string {
  return (bytes / (1024 * 1024)).toFixed(1);
}

async function refreshCacheInfo() {
  await audioCache.init();
  await audioCache.refreshSize();
  cacheSize.value = audioCache.currentSize;
  cacheEntryCount.value = await audioCache.getEntryCount();
}

async function clearCache() {
  const freed = cacheSize.value;
  await audioCache.clear();
  await refreshCacheInfo();
  message.success(`缓存已清理，释放 ${formatMB(freed)} MB`);
}

async function checkTools() {
  try {
    const result = await settingsStore.checkTools();
    ffmpegOk.value = result.ffmpeg;
  } catch {
    ffmpegOk.value = false;
  }
}

function goBack() {
  router.push("/");
}

function setTheme(t: "light" | "dark" | "system", e: MouseEvent) {
  setThemeClickOrigin(e.clientX, e.clientY);
  settingsStore.setTheme(t);
}

function setDownloadFormat(fmt: AudioFormat) {
  settingsStore.downloadFormat = fmt;
  settingsStore.saveSettings();
}

function setDownloadQuality(q: AudioQuality) {
  settingsStore.downloadQuality = q;
  settingsStore.saveSettings();
}

async function setMinimizeToTray(val: boolean) {
  settingsStore.minimizeToTray = val;
  await settingsStore.saveSettings();
}

async function setAutostartEnabled(val: boolean) {
  await settingsStore.setAutostartEnabled(val);
}

async function setAccentColor(color: string) {
  settingsStore.accentColor = color;
  await settingsStore.saveSettings();
}

async function setCustomAccentColor(e: Event) {
  const target = e.target as HTMLInputElement;
  if (target.value) {
    await setAccentColor(target.value);
  }
}

async function setDesktopLyricsEnabled(val: boolean) {
  settingsStore.desktopLyricsEnabled = val;
  await settingsStore.saveSettings();
  if (val) {
    const { useWindowManager } = await import("@/composables/useWindowManager");
    useWindowManager().showDesktopLyrics();
  } else {
    const { useWindowManager } = await import("@/composables/useWindowManager");
    useWindowManager().hideDesktopLyrics();
  }
}

async function setDesktopLyricsFontSize(size: number) {
  settingsStore.desktopLyricsFontSize = size;
  await settingsStore.saveSettings();
}

async function setDesktopLyricsLocked(val: boolean) {
  settingsStore.desktopLyricsLocked = val;
  await settingsStore.saveSettings();
}

onMounted(() => {
  checkTools();
  refreshCacheInfo();
});
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <button class="back-btn" @click="goBack">
        <NIcon size="20"><ArrowBackOutline /></NIcon>
        返回
      </button>
      <h2>设置</h2>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <h3 class="section-title">外观</h3>
        <div class="setting-item">
          <span class="setting-label">主题</span>
          <div class="theme-options">
            <button
              class="theme-btn"
              :class="{ active: settingsStore.theme === 'light' }"
              @click="setTheme('light', $event)"
            >
              亮色
            </button>
            <button
              class="theme-btn"
              :class="{ active: settingsStore.theme === 'dark' }"
              @click="setTheme('dark', $event)"
            >
              暗色
            </button>
            <button
              class="theme-btn"
              :class="{ active: settingsStore.theme === 'system' }"
              @click="setTheme('system', $event)"
            >
              跟随系统
            </button>
          </div>
        </div>
        <div class="setting-item">
          <span class="setting-label">主题色</span>
          <div class="accent-color-section">
            <div class="preset-colors">
              <button
                v-for="preset in PRESET_COLORS"
                :key="preset.color"
                class="color-dot"
                :class="{ active: settingsStore.accentColor === preset.color }"
                :style="{ backgroundColor: preset.color }"
                :title="preset.name"
                @click="setAccentColor(preset.color)"
              >
                <CheckmarkCircleOutline v-if="settingsStore.accentColor === preset.color" class="color-check" />
              </button>
            </div>
            <label class="custom-color-btn" title="自定义颜色">
              <span class="custom-color-preview" :style="{ backgroundColor: settingsStore.accentColor }"></span>
              <input
                type="color"
                :value="settingsStore.accentColor"
                class="color-input"
                @input="setCustomAccentColor"
              />
            </label>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">下载</h3>
        <div class="setting-item">
          <span class="setting-label">输出目录</span>
          <div class="setting-control">
            <NInput
              :value="settingsStore.outputDir"
              readonly
              size="small"
              style="flex: 1"
            />
            <NButton size="small" @click="settingsStore.pickDirectory()">
              <template #icon>
                <NIcon><FolderOpenOutline /></NIcon>
              </template>
              选择
            </NButton>
          </div>
        </div>
        <div class="setting-item">
          <span class="setting-label">音频格式</span>
          <div class="btn-group">
            <button
              v-for="fmt in (['mp3', 'flac', 'wav', 'aac'] as const)"
              :key="fmt"
              class="option-btn"
              :class="{ active: settingsStore.downloadFormat === fmt }"
              @click="setDownloadFormat(fmt)"
            >{{ fmt.toUpperCase() }}</button>
          </div>
        </div>
        <div class="setting-item">
          <span class="setting-label">音质</span>
          <div class="btn-group">
            <button
              v-for="q in (['high', 'medium', 'low'] as const)"
              :key="q"
              class="option-btn"
              :class="{ active: settingsStore.downloadQuality === q }"
              @click="setDownloadQuality(q)"
            >{{ { high: '高', medium: '中', low: '低' }[q] }}</button>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">系统</h3>
        <div class="setting-item">
          <span class="setting-label">关闭时最小化到托盘</span>
          <NSwitch
            :value="settingsStore.minimizeToTray"
            @update:value="setMinimizeToTray"
          />
        </div>
        <div class="setting-item">
          <span class="setting-label">开机自启</span>
          <NSwitch
            :value="settingsStore.autostartEnabled"
            @update:value="setAutostartEnabled"
          />
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">账号</h3>
        <div class="setting-item column">
          <div class="slider-header">
            <span class="setting-label">B站 SESSDATA</span>
            <button class="sessdata-toggle" @click="showSessdata = !showSessdata">
              <NIcon size="14">
                <EyeOffOutline v-if="!showSessdata" />
                <EyeOutline v-else />
              </NIcon>
            </button>
          </div>
          <NInput
            :type="showSessdata ? 'text' : 'password'"
            :value="settingsStore.sessdata"
            placeholder="粘贴你的 SESSDATA Cookie"
            size="small"
            @update:value="setSessdata"
          />
          <p class="sessdata-hint">
            用于获取完整评论列表等功能。可在浏览器登录 B站后，从 Cookie 中复制 SESSDATA 的值。留空则使用未登录状态。
          </p>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">桌面歌词</h3>
        <div class="setting-item">
          <span class="setting-label">启用桌面歌词</span>
          <NSwitch
            :value="settingsStore.desktopLyricsEnabled"
            @update:value="setDesktopLyricsEnabled"
          />
        </div>
        <div class="setting-item column">
          <div class="slider-header">
            <span class="setting-label">歌词字号</span>
            <span class="slider-value">{{ settingsStore.desktopLyricsFontSize }}px</span>
          </div>
          <input
            type="range"
            class="font-size-slider"
            min="10"
            max="50"
            :value="settingsStore.desktopLyricsFontSize"
            @input="setDesktopLyricsFontSize(Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="setting-item">
          <span class="setting-label">锁定歌词位置</span>
          <NSwitch
            :value="settingsStore.desktopLyricsLocked"
            @update:value="setDesktopLyricsLocked"
          />
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">数据统计</h3>
        <div v-if="historyStore.history.length === 0" class="empty-hint">
          暂无播放记录
        </div>
        <template v-else>
          <div class="stat-row">
            <span class="stat-label">总播放次数</span>
            <span class="stat-value">{{ totalPlays.toLocaleString() }} 次</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">收听歌曲数</span>
            <span class="stat-value">{{ uniqueSongs }} 首</span>
          </div>
          <div class="stat-divider"></div>
          <p class="stat-subtitle">最常听的歌曲</p>
          <div
            v-for="(entry, i) in topSongs"
            :key="entry.song.bvid"
            class="top-song-item"
          >
            <span class="top-song-rank">{{ i + 1 }}</span>
            <span class="top-song-name">{{ entry.song.title }} - {{ entry.song.author }}</span>
            <span class="top-song-count">{{ entry.playCount }} 次</span>
          </div>
        </template>
      </div>

      <div class="settings-section">
        <h3 class="section-title">存储管理</h3>
        <div class="stat-row">
          <span class="stat-label">音频缓存</span>
          <span class="stat-value">{{ formatMB(cacheSize) }} MB / {{ formatMB(MAX_CACHE_SIZE) }} MB</span>
        </div>
        <div class="cache-progress">
          <div class="cache-progress-bar" :style="{ width: cachePercent + '%' }"></div>
        </div>
        <div class="stat-row">
          <span class="stat-label">缓存条目</span>
          <span class="stat-value">{{ cacheEntryCount }} 条</span>
        </div>
        <div class="clear-cache-row">
          <NButton size="small" type="warning" @click="clearCache">
            <template #icon>
              <NIcon><TrashOutline /></NIcon>
            </template>
            清理缓存
          </NButton>
        </div>
      </div>

      <div class="settings-section">
        <h3 class="section-title">内置工具</h3>
        <div class="setting-item">
          <span class="setting-label">ffmpeg</span>
          <NTag :type="ffmpegOk ? 'success' : 'error'" size="small">
            <template #icon>
              <NIcon>
                <CheckmarkCircleOutline v-if="ffmpegOk" />
                <CloseCircleOutline v-else />
              </NIcon>
            </template>
            {{ ffmpegOk ? "正常" : "异常" }}
          </NTag>
        </div>
        <div class="setting-item">
          <span class="setting-label"></span>
          <NButton size="small" @click="checkTools">
            <template #icon>
              <NIcon><RefreshOutline /></NIcon>
            </template>
            重新检测
          </NButton>
        </div>
        <p class="tool-hint">
          所有工具已内置，无需额外安装。
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.settings-header h2 {
  font-size: 20px;
  font-weight: 600;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-secondary);
}

.back-btn:hover {
  background: var(--card-hover);
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 16px;
  border: 1px solid var(--border-color);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0;
}

.setting-item + .setting-item {
  border-top: 1px solid var(--border-color);
}

.setting-label {
  font-size: 14px;
}

.setting-control {
  display: flex;
  gap: 8px;
  align-items: center;
  flex: 1;
  margin-left: 20px;
  max-width: 300px;
}

.theme-options {
  display: flex;
  gap: 6px;
}

.theme-btn {
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.theme-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.theme-btn.active {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}

.btn-group {
  display: flex;
  gap: 4px;
}

.option-btn {
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  cursor: pointer;
}

.option-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.option-btn.active {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}

.setting-item.column {
  flex-direction: column;
  align-items: stretch;
}

.slider-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.slider-value {
  font-size: 13px;
  color: var(--accent-color);
  font-weight: 500;
}

.font-size-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: var(--border-color);
  outline: none;
  margin-top: 8px;
  cursor: pointer;
}

.font-size-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent-color);
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
}

.tool-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.6;
}

.empty-hint {
  font-size: 13px;
  color: var(--text-tertiary);
  text-align: center;
  padding: 16px 0;
}

.stat-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.stat-row + .stat-row {
  border-top: 1px solid var(--border-color);
}

.stat-label {
  font-size: 14px;
}

.stat-value {
  font-size: 14px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.stat-divider {
  height: 1px;
  background: var(--border-color);
  margin: 8px 0;
}

.stat-subtitle {
  font-size: 13px;
  font-weight: 600;
  margin: 8px 0 4px;
}

.top-song-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  font-size: 13px;
}

.top-song-rank {
  min-width: 24px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-align: center;
}

.top-song-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.top-song-count {
  color: var(--accent-color);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.cache-progress {
  height: 6px;
  background: var(--border-color);
  border-radius: 3px;
  overflow: hidden;
  margin: 4px 0;
}

.cache-progress-bar {
  height: 100%;
  background: var(--accent-color);
  border-radius: 3px;
  transition: width 0.3s ease;
  min-width: 0;
}

.clear-cache-row {
  display: flex;
  justify-content: center;
  margin-top: 12px;
}

.accent-color-section {
  display: flex;
  align-items: center;
  gap: 10px;
}

.preset-colors {
  display: flex;
  gap: 6px;
}

.color-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.15s;
}

.color-dot:hover {
  transform: scale(1.1);
}

.color-dot.active {
  border-color: var(--app-text);
}

.color-check {
  font-size: 14px;
  color: white;
}

.custom-color-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  overflow: hidden;
  cursor: pointer;
  border: 2px dashed var(--border-color);
  position: relative;
}

.custom-color-preview {
  position: absolute;
  inset: 0;
}

.color-input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
  width: 100%;
  height: 100%;
}

.sessdata-toggle {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
}

.sessdata-toggle:hover {
  background: var(--card-hover);
  color: var(--text-secondary);
}

.sessdata-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.6;
}
</style>
