import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { DownloadProgress, Song } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { useSettingsStore } from "@/stores/settings";

export interface QueuedSong {
  bvid: string;
  title: string;
}

export const useDownloadStore = defineStore("download", () => {
  const tasks = ref<Map<string, DownloadProgress>>(new Map());
  const queue = ref<QueuedSong[]>([]);
  const maxConcurrent = 2;
  const maxRetries = 3;
  const retryDelay = 3000; // 3s
  let unlisten: (() => void) | null = null;

  // track retries per bvid
  const retryCount = new Map<string, number>();

  const activeCount = computed(() => {
    let count = 0;
    for (const task of tasks.value.values()) {
      if (task.status === "downloading" || task.status === "converting") {
        count++;
      }
    }
    return count;
  });

  const hasActiveTasks = computed(
    () => activeCount.value > 0 || queue.value.length > 0
  );

  const queueInfo = computed(() => ({
    active: activeCount.value,
    queued: queue.value.length,
  }));

  let notificationPermission = false;

  async function ensureNotificationPermission() {
    if (notificationPermission) return true;
    const granted = await isPermissionGranted();
    if (granted) {
      notificationPermission = true;
      return true;
    }
    const permission = await requestPermission();
    notificationPermission = permission === "granted";
    return notificationPermission;
  }

  async function notifyDownloadDone(task: DownloadProgress) {
    const ok = await ensureNotificationPermission();
    if (!ok) return;
    sendNotification({
      title: "下载完成",
      body: task.bvid,
    });
  }

  async function startListening() {
    if (unlisten) return;
    unlisten = await listen<DownloadProgress>(
      "download-progress",
      (event) => {
        const prev = tasks.value.get(event.payload.taskId);
        tasks.value.set(event.payload.taskId, event.payload);
        // If a task just finished, process the queue
        const wasActive =
          prev?.status === "downloading" || prev?.status === "converting";
        const nowDone =
          event.payload.status === "done" ||
          event.payload.status === "error";
        if (wasActive && nowDone) {
          if (event.payload.status === "done") {
            notifyDownloadDone(event.payload);
          }
          if (event.payload.status === "error") {
            scheduleRetry(event.payload.bvid);
          }
          processQueue();
        }
      }
    );
  }

  function stopListening() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }

  async function startDownload(bvid: string, outputDir?: string) {
    await startListening();
    try {
      const settings = useSettingsStore();
      const taskId = await invoke<string>("start_download", {
        bvid,
        options: {
          outputDir: outputDir || undefined,
          format: settings.downloadFormat,
          quality: settings.downloadQuality,
        },
      });
      tasks.value.set(taskId, {
        taskId,
        bvid,
        status: "pending",
        progress: 0,
        downloadedBytes: 0,
        totalBytes: 0,
        downloadedText: "0 B",
        totalText: "0 B",
      });
      return taskId;
    } catch (e: any) {
      throw new Error(
        typeof e === "string" ? e : e.message || "Download failed"
      );
    }
  }

  function processQueue() {
    while (activeCount.value < maxConcurrent && queue.value.length > 0) {
      const item = queue.value.shift()!;
      startDownload(item.bvid).catch(() => {
        scheduleRetry(item.bvid);
      });
    }
  }

  function scheduleRetry(bvid: string) {
    const count = (retryCount.get(bvid) || 0) + 1;
    if (count > maxRetries) {
      retryCount.delete(bvid);
      return;
    }
    retryCount.set(bvid, count);
    setTimeout(() => {
      // bvid may already be downloading again (e.g. user re-triggered)
      const alreadyActive = [...tasks.value.values()].some(
        (t) => t.bvid === bvid && (t.status === "downloading" || t.status === "converting" || t.status === "pending")
      );
      if (!alreadyActive) {
        startDownload(bvid).catch(() => {
          scheduleRetry(bvid);
        });
      }
    }, retryDelay * count);
  }

  function batchDownload(songs: Song[]) {
    const activeBvids = new Set<string>();
    for (const task of tasks.value.values()) {
      if (
        task.status === "downloading" ||
        task.status === "converting" ||
        task.status === "pending" ||
        task.status === "done"
      ) {
        activeBvids.add(task.bvid);
      }
    }
    const queuedBvids = new Set(queue.value.map((q) => q.bvid));

    for (const song of songs) {
      if (!activeBvids.has(song.bvid) && !queuedBvids.has(song.bvid)) {
        queue.value.push({ bvid: song.bvid, title: song.title });
      }
    }
    processQueue();
  }

  async function openInExplorer(filePath: string) {
    await invoke("open_in_explorer", { path: filePath });
  }

  function getTask(taskId: string): DownloadProgress | undefined {
    return tasks.value.get(taskId);
  }

  function clearCompleted() {
    for (const [id, task] of tasks.value) {
      if (task.status === "done") {
        retryCount.delete(task.bvid);
        tasks.value.delete(id);
      } else if (task.status === "error") {
        // only clear if retries are exhausted
        if ((retryCount.get(task.bvid) || 0) >= maxRetries) {
          retryCount.delete(task.bvid);
          tasks.value.delete(id);
        }
      }
    }
  }

  return {
    tasks,
    queue,
    activeCount,
    hasActiveTasks,
    queueInfo,
    startListening,
    stopListening,
    startDownload,
    batchDownload,
    openInExplorer,
    getTask,
    clearCompleted,
  };
});
