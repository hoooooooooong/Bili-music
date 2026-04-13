import { defineStore } from "pinia";
import { ref } from "vue";
import type { DownloadProgress } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const useDownloadStore = defineStore("download", () => {
  const tasks = ref<Map<string, DownloadProgress>>(new Map());
  let unlisten: (() => void) | null = null;

  async function startListening() {
    if (unlisten) return;
    unlisten = await listen<DownloadProgress>(
      "download-progress",
      (event) => {
        tasks.value.set(event.payload.taskId, event.payload);
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
      const taskId = await invoke<string>("start_download", {
        bvid,
        options: outputDir ? { outputDir } : null,
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

  async function openInExplorer(filePath: string) {
    await invoke("open_in_explorer", { path: filePath });
  }

  function getTask(taskId: string): DownloadProgress | undefined {
    return tasks.value.get(taskId);
  }

  function clearCompleted() {
    for (const [id, task] of tasks.value) {
      if (task.status === "done" || task.status === "error") {
        tasks.value.delete(id);
      }
    }
  }

  return {
    tasks,
    startListening,
    stopListening,
    startDownload,
    openInExplorer,
    getTask,
    clearCompleted,
  };
});
