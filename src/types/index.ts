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
  accentColor?: string;
}

export interface WindowGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized: boolean;
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

export interface FavoritesFolder {
  id: number;
  title: string;
  cover: string;
  mediaCount: number;
}

export interface FavoritesFolderListResponse {
  folders: FavoritesFolder[];
  uid: string;
}

export interface MediaResourcePage {
  videos: Song[];
  page: number;
  total: number;
  hasMore: boolean;
}

export interface CommentMember {
  name: string;
  avatar: string;
  level: number;
}

export interface Comment {
  rpid: number;
  message: string;
  like: number;
  rcount: number;
  member: CommentMember;
  ctime: number;
}

export interface CommentResponse {
  comments: Comment[];
  isEnd: boolean;
}
