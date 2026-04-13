<script setup lang="ts">
import { computed, onMounted } from "vue";
import { NIcon, NProgress, NButton, useMessage } from "naive-ui";
import {
  ArrowBackOutline,
  TrashOutline,
  FolderOpenOutline,
} from "@vicons/ionicons5";
import { useRouter } from "vue-router";
import { useDownloadStore } from "@/stores/download";
import type { DownloadProgress } from "@/types";

const router = useRouter();
const downloadStore = useDownloadStore();
const message = useMessage();

const taskList = computed(() =>
  Array.from(downloadStore.tasks.values()).reverse()
);

const activeTaskList = computed(() =>
  taskList.value.filter(
    (t) => t.status === "downloading" || t.status === "converting"
  )
);

const queuedList = computed(() => downloadStore.queue);

function goBack() {
  router.push("/");
}

async function openFile(task: DownloadProgress) {
  if (task.filePath) {
    try {
      await downloadStore.openInExplorer(task.filePath);
    } catch {
      message.error("打开目录失败");
    }
  }
}

function statusText(status: string): string {
  switch (status) {
    case "pending": return "等待中";
    case "downloading": return "下载中";
    case "converting": return "转换中";
    case "done": return "已完成";
    case "error": return "失败";
    default: return status;
  }
}

onMounted(() => {
  downloadStore.startListening();
});
</script>

<template>
  <div class="download-page">
    <div class="page-header">
      <button class="back-btn" @click="goBack">
        <NIcon size="20"><ArrowBackOutline /></NIcon>
        返回
      </button>
      <h2>下载管理</h2>
      <NButton
        v-if="taskList.length > 0"
        size="small"
        quaternary
        @click="downloadStore.clearCompleted()"
      >
        <template #icon><NIcon><TrashOutline /></NIcon></template>
        清除已完成
      </NButton>
    </div>

    <div class="download-list">
      <!-- Queue status bar -->
      <div v-if="downloadStore.hasActiveTasks" class="queue-status">
        <span v-if="activeTaskList.length > 0" class="queue-active">
          正在下载 {{ activeTaskList.length }} 首
        </span>
        <span v-if="queuedList.length > 0" class="queue-queued">
          ，队列中 {{ queuedList.length }} 首
        </span>
      </div>
      <div v-if="taskList.length === 0" class="empty-state">
        <p>暂无下载任务</p>
      </div>

      <div v-for="task in taskList" :key="task.taskId" class="download-item">
        <div class="item-header">
          <span class="item-bvid">{{ task.bvid }}</span>
          <span class="item-status" :class="`status-${task.status}`">
            {{ statusText(task.status) }}
          </span>
        </div>

        <NProgress
          v-if="task.status === 'downloading'"
          type="line"
          :percentage="task.progress"
          :indicator-placement="'inside'"
          :style="{ marginBottom: '8px' }"
        />

        <div v-if="task.status === 'downloading'" class="item-detail">
          {{ task.downloadedText }} / {{ task.totalText }}
        </div>

        <div v-if="task.status === 'done'" class="item-actions">
          <span class="item-filename">{{ task.fileName }}</span>
          <NButton size="tiny" @click="openFile(task)">
            <template #icon><NIcon><FolderOpenOutline /></NIcon></template>
            打开目录
          </NButton>
        </div>

        <div v-if="task.status === 'error'" class="item-error">
          {{ task.errorMessage || "未知错误" }}
        </div>

        <div v-if="task.status === 'converting'" class="item-detail">
          正在转换{{ task.fileName ? `为 ${task.fileName.split('.').pop()?.toUpperCase()}` : '' }}...
        </div>
      </div>

      <!-- Queued items -->
      <div v-for="item in queuedList" :key="'q-' + item.bvid" class="download-item queued-item">
        <div class="item-header">
          <span class="item-bvid">{{ item.title || item.bvid }}</span>
          <span class="item-status status-pending">等待中</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.download-page {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}

.page-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.page-header h2 {
  flex: 1;
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

.download-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.queue-status {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 12px;
  background: var(--accent-light);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.queue-active {
  color: var(--accent-color);
  font-weight: 500;
}

.queue-queued {
  color: var(--text-secondary);
}

.queued-item {
  opacity: 0.7;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.download-item {
  background: var(--card-bg);
  border-radius: 10px;
  padding: 14px 16px;
  border: 1px solid var(--border-color);
}

.item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.item-bvid {
  font-size: 13px;
  font-weight: 500;
}

.item-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pending {
  background: var(--card-hover);
  color: var(--text-secondary);
}

.status-downloading,
.status-converting {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.status-done {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.item-detail {
  font-size: 12px;
  color: var(--text-secondary);
}

.item-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.item-filename {
  font-size: 12px;
  color: var(--text-secondary);
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-error {
  font-size: 12px;
  color: #ef4444;
}
</style>
