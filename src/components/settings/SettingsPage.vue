<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  NIcon,
  NButton,
  NInput,
  NTag,
  useMessage,
} from "naive-ui";
import {
  ArrowBackOutline,
  FolderOpenOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  RefreshOutline,
} from "@vicons/ionicons5";
import { useSettingsStore } from "@/stores/settings";
import { useRouter } from "vue-router";

const router = useRouter();
const settingsStore = useSettingsStore();
const message = useMessage();

const ffmpegOk = ref(false);

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

function setTheme(t: "light" | "dark" | "system") {
  settingsStore.setTheme(t);
}

onMounted(() => {
  checkTools();
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
              @click="setTheme('light')"
            >
              亮色
            </button>
            <button
              class="theme-btn"
              :class="{ active: settingsStore.theme === 'dark' }"
              @click="setTheme('dark')"
            >
              暗色
            </button>
            <button
              class="theme-btn"
              :class="{ active: settingsStore.theme === 'system' }"
              @click="setTheme('system')"
            >
              跟随系统
            </button>
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

.tool-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.6;
}
</style>
