export interface Song {
  bvid: string;
  title: string;
  author: string;
  duration: string;
  playCount: number;
  playCountText: string;
  coverUrl: string;
  description?: string;
}

export interface SearchResponse {
  results: Song[];
  page: number;
  total: number;
  pageSize: number;
}

export interface LyricLine {
  time: number;
  text: string;
}

export interface LyricsData {
  lyrics: LyricLine[];
  song?: string;
  artist?: string;
}

export interface DownloadProgress {
  taskId: string;
  bvid: string;
  status: "pending" | "downloading" | "converting" | "done" | "error";
  progress: number;
  downloadedBytes: number;
  totalBytes: number;
  downloadedText: string;
  totalText: string;
  filePath?: string;
  fileName?: string;
  errorMessage?: string;
}

export interface AppSettings {
  outputDir: string;
  theme: "light" | "dark" | "system";
  cacheSize: number;
  volume: number;
  downloadFormat: AudioFormat;
  downloadQuality: AudioQuality;
  minimizeToTray?: boolean;
  autostartEnabled?: boolean;
}

export type AudioFormat = "mp3" | "flac" | "wav" | "aac";
export type AudioQuality = "high" | "medium" | "low";

export interface AudioUrlInfo {
  url: string;
  ext: string;
  filesize: number;
}

export interface Playlist {
  id: string;
  name: string;
  createdAt: number;
  songs: Song[];
  coverUrl?: string;
}

export type PlayMode = "sequential" | "loop" | "random";
