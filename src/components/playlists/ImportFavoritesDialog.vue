<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NIcon, NCheckbox, NProgress, NSpin } from "naive-ui";
import {
  CloseOutline,
  SearchOutline,
  CheckmarkCircleOutline,
  CloudDownloadOutline,
} from "@vicons/ionicons5";
import { usePlaylistStore } from "@/stores/playlists";
import { useMessage } from "naive-ui";
import type {
  FavoritesFolder,
  FavoritesFolderListResponse,
  MediaResourcePage,
  Song,
} from "@/types";

const emit = defineEmits<{
  close: [];
}>();

const playlistStore = usePlaylistStore();
const message = useMessage();

type Step = "input" | "select" | "importing" | "done";
const step = ref<Step>("input");

const uid = ref("");
const uidLoading = ref(false);
const uidError = ref("");
const folders = ref<FavoritesFolder[]>([]);
const selectedFolderIds = ref<Set<number>>(new Set());

const importingFolder = ref("");
const currentPage = ref(1);
const totalImported = ref(0);
const totalExpected = ref(0);
const importCancelled = ref(false);
const importError = ref("");

const doneSummary = ref<{ name: string; count: number }[]>([]);

const canProceed = computed(() => selectedFolderIds.value.size > 0);

async function searchFolders() {
  uidError.value = "";
  if (!uid.value.trim()) {
    uidError.value = "请输入 UID";
    return;
  }
  uidLoading.value = true;
  try {
    const result = await invoke<FavoritesFolderListResponse>(
      "fetch_user_favorites_folders",
      { uid: uid.value.trim() }
    );
    folders.value = result.folders;
    step.value = "select";
  } catch (e: any) {
    uidError.value = e?.toString?.() || "获取收藏夹失败";
  } finally {
    uidLoading.value = false;
  }
}

function toggleFolder(id: number) {
  if (selectedFolderIds.value.has(id)) {
    selectedFolderIds.value.delete(id);
  } else {
    selectedFolderIds.value.add(id);
  }
  // Trigger reactivity
  selectedFolderIds.value = new Set(selectedFolderIds.value);
}

function selectAllFolders() {
  if (selectedFolderIds.value.size === folders.value.length) {
    selectedFolderIds.value = new Set();
  } else {
    selectedFolderIds.value = new Set(folders.value.map((f) => f.id));
  }
}

const allSelected = computed(
  () => selectedFolderIds.value.size === folders.value.length && folders.value.length > 0
);

function goBack() {
  if (step.value === "select") {
    step.value = "input";
  }
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function startImport() {
  step.value = "importing";
  importCancelled.value = false;
  importError.value = "";
  doneSummary.value = [];
  totalImported.value = 0;

  const selectedFolders = folders.value.filter((f) =>
    selectedFolderIds.value.has(f.id)
  );
  totalExpected.value = selectedFolders.reduce((sum, f) => sum + f.mediaCount, 0);

  for (const folder of selectedFolders) {
    if (importCancelled.value) break;

    importingFolder.value = folder.title;
    currentPage.value = 1;

    const playlist = playlistStore.createPlaylist(folder.title);
    let folderImported = 0;
    const MAX_SONGS = 1000;

    try {
      while (!importCancelled.value && folderImported < MAX_SONGS) {
        const result = await invoke<MediaResourcePage>(
          "fetch_favorites_folder_videos",
          { uid: uid.value.trim(), mediaId: folder.id, page: currentPage.value }
        );

        const videos: Song[] = result.videos;
        if (videos.length === 0) break;

        for (const video of videos) {
          if (importCancelled.value) break;
          if (folderImported >= MAX_SONGS) break;
          playlistStore.addSong(playlist.id, video);
          folderImported++;
          totalImported.value++;
        }

        if (!result.hasMore || importCancelled.value) break;
        currentPage.value++;
        // Rate limit: 300ms between pages
        await sleep(300);
      }

      doneSummary.value.push({ name: folder.title, count: folderImported });
    } catch (e: any) {
      importError.value = `导入「${folder.title}」时出错: ${e?.toString?.() || "未知错误"}`;
      doneSummary.value.push({ name: folder.title, count: folderImported });
      // Continue with next folder
    }
  }

  step.value = "done";
}

function cancelImport() {
  importCancelled.value = true;
}

function handleClose() {
  emit("close");
}
</script>

<template>
  <div class="dialog-overlay" @click.self="handleClose">
    <div class="dialog-box">
      <button class="dialog-close" @click="handleClose">
        <NIcon size="20"><CloseOutline /></NIcon>
      </button>

      <!-- Step 1: Input UID -->
      <template v-if="step === 'input'">
        <div class="dialog-header">
          <h3>导入B站收藏夹</h3>
          <p class="dialog-desc">输入 Bilibili 用户的 UID，导入其公开收藏夹为本地歌单</p>
        </div>
        <div class="uid-input-row">
          <input
            v-model="uid"
            class="uid-input"
            placeholder="请输入 UID（纯数字）"
            :disabled="uidLoading"
            @keyup.enter="searchFolders"
          />
          <button
            class="search-btn"
            :disabled="uidLoading || !uid.trim()"
            @click="searchFolders"
          >
            <NSpin v-if="uidLoading" :size="16" />
            <NIcon v-else size="18"><SearchOutline /></NIcon>
          </button>
        </div>
        <p v-if="uidError" class="error-text">{{ uidError }}</p>
        <p class="hint-text">
          在 B站 个人主页 URL 中可以找到 UID，例如 bilibili.com/12345 中的 12345
        </p>
      </template>

      <!-- Step 2: Select folders -->
      <template v-else-if="step === 'select'">
        <div class="dialog-header">
          <h3>选择收藏夹</h3>
          <p class="dialog-desc">共找到 {{ folders.length }} 个公开收藏夹</p>
        </div>
        <div class="folder-list">
          <label
            v-for="folder in folders"
            :key="folder.id"
            class="folder-item"
            :class="{ selected: selectedFolderIds.has(folder.id) }"
          >
            <NCheckbox
              :checked="selectedFolderIds.has(folder.id)"
              @update:checked="toggleFolder(folder.id)"
            />
            <img v-if="folder.cover" :src="folder.cover" class="folder-cover" loading="lazy" />
            <div v-else class="folder-cover-placeholder">{{ folder.title.charAt(0) }}</div>
            <div class="folder-info">
              <span class="folder-name">{{ folder.title }}</span>
              <span class="folder-count">{{ folder.mediaCount }} 个视频</span>
            </div>
          </label>
        </div>
        <div class="dialog-actions">
          <button class="btn-secondary" @click="goBack">返回</button>
          <label class="select-all-label">
            <NCheckbox :checked="allSelected" @update:checked="selectAllFolders" />
            <span>全选</span>
          </label>
          <button class="btn-primary" :disabled="!canProceed" @click="startImport">
            导入选中 ({{ selectedFolderIds.size }})
          </button>
        </div>
      </template>

      <!-- Step 3: Importing -->
      <template v-else-if="step === 'importing'">
        <div class="dialog-header">
          <h3>正在导入</h3>
          <p class="dialog-desc">
            正在导入「{{ importingFolder }}」第 {{ currentPage }} 页
          </p>
        </div>
        <div class="import-progress">
          <NProgress
            type="line"
            :percentage="
              totalExpected > 0 ? Math.round((totalImported / totalExpected) * 100) : 0
            "
            :show-indicator="true"
          />
          <p class="import-stats">
            已导入 {{ totalImported }} / {{ totalExpected }} 首
          </p>
        </div>
        <div class="dialog-actions">
          <button class="btn-danger" @click="cancelImport">取消导入</button>
        </div>
      </template>

      <!-- Step 4: Done -->
      <template v-else-if="step === 'done'">
        <div class="dialog-header done-header">
          <NIcon size="48" color="var(--accent-color)"><CheckmarkCircleOutline /></NIcon>
          <h3>导入完成</h3>
        </div>
        <div v-if="importError" class="error-text import-error">{{ importError }}</div>
        <div class="done-summary">
          <div v-for="item in doneSummary" :key="item.name" class="summary-item">
            <CloudDownloadOutline class="summary-icon" />
            <span class="summary-name">{{ item.name }}</span>
            <span class="summary-count">{{ item.count }} 首</span>
          </div>
        </div>
        <p v-if="importCancelled && totalImported < totalExpected" class="hint-text">
          已取消，已导入的歌单已保留
        </p>
        <div class="dialog-actions">
          <button class="btn-primary" @click="handleClose">完成</button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-box {
  background: var(--card-bg);
  border-radius: 14px;
  width: 460px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
  position: relative;
  padding: 24px;
}

.dialog-close {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--text-secondary);
  z-index: 1;
}

.dialog-close:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.dialog-header {
  margin-bottom: 20px;
}

.dialog-header h3 {
  font-size: 17px;
  font-weight: 600;
  color: var(--app-text);
  margin-bottom: 4px;
}

.dialog-desc {
  font-size: 13px;
  color: var(--text-secondary);
}

.done-header {
  text-align: center;
  padding-top: 8px;
}

.done-header h3 {
  margin-top: 8px;
}

.uid-input-row {
  display: flex;
  gap: 8px;
}

.uid-input {
  flex: 1;
  height: 38px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--app-bg);
  font-size: 14px;
  color: var(--app-text);
}

.uid-input:focus {
  border-color: var(--accent-color);
}

.uid-input::placeholder {
  color: var(--text-tertiary);
}

.search-btn {
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-color);
  border-radius: 8px;
  color: #fff;
  flex-shrink: 0;
}

.search-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-text {
  font-size: 13px;
  color: #e53935;
  margin-top: 8px;
}

.import-error {
  margin-bottom: 12px;
}

.hint-text {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 12px;
  line-height: 1.5;
}

.folder-list {
  max-height: 320px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.folder-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.folder-item:hover {
  background: var(--card-hover);
}

.folder-item.selected {
  background: var(--accent-light);
}

.folder-cover {
  width: 40px;
  height: 40px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
}

.folder-cover-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 6px;
  background: var(--card-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.folder-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.folder-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

.dialog-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
  flex-shrink: 0;
}

.select-all-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  margin-right: auto;
}

.btn-primary {
  height: 34px;
  padding: 0 18px;
  background: var(--accent-color);
  color: #fff;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  height: 34px;
  padding: 0 14px;
  background: var(--card-hover);
  color: var(--app-text);
  border-radius: 8px;
  font-size: 14px;
}

.btn-secondary:hover {
  background: var(--border-color);
}

.btn-danger {
  height: 34px;
  padding: 0 14px;
  background: transparent;
  color: #e53935;
  border: 1px solid #e53935;
  border-radius: 8px;
  font-size: 14px;
}

.btn-danger:hover {
  background: rgba(229, 57, 53, 0.08);
}

.import-progress {
  margin-bottom: 16px;
}

.import-stats {
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
  margin-top: 8px;
}

.done-summary {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 8px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--app-bg);
  border-radius: 8px;
}

.summary-icon {
  color: var(--accent-color);
  font-size: 18px;
  flex-shrink: 0;
}

.summary-name {
  flex: 1;
  font-size: 14px;
  color: var(--app-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-count {
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}
</style>
