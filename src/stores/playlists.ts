import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Song, Playlist } from "@/types";
import { useHistoryStore } from "./history";

export interface ExportedPlaylist {
  name: string;
  songs: Pick<Song, "bvid" | "title" | "author" | "duration" | "playCount" | "playCountText" | "coverUrl">[];
}

export interface ExportData {
  version: 1;
  exportedAt: string;
  playlists: ExportedPlaylist[];
}

export const usePlaylistStore = defineStore(
  "playlists",
  () => {
    const playlists = ref<Playlist[]>([]);

    const smartPlaylists = computed<Playlist[]>(() => {
      const history = useHistoryStore().history;
      const mostPlayed = [...history]
        .sort((a, b) => b.playCount - a.playCount)
        .slice(0, 50)
        .map((h) => h.song);
      const recentlyPlayed = [...history]
        .sort((a, b) => b.playedAt - a.playedAt)
        .slice(0, 50)
        .map((h) => h.song);
      return [
        { id: "__smart_most_played__", name: "最常播放", createdAt: 0, songs: mostPlayed, coverUrl: undefined },
        { id: "__smart_recently_played__", name: "最近播放", createdAt: 0, songs: recentlyPlayed, coverUrl: undefined },
      ];
    });

    function createPlaylist(name: string): Playlist {
      const playlist: Playlist = {
        id: crypto.randomUUID(),
        name,
        createdAt: Date.now(),
        songs: [],
      };
      playlists.value.push(playlist);
      return playlist;
    }

    function deletePlaylist(id: string) {
      const idx = playlists.value.findIndex((p) => p.id === id);
      if (idx >= 0) {
        playlists.value.splice(idx, 1);
      }
    }

    function renamePlaylist(id: string, name: string) {
      const playlist = playlists.value.find((p) => p.id === id);
      if (playlist) {
        playlist.name = name;
      }
    }

    function addSong(playlistId: string, song: Song) {
      const playlist = playlists.value.find((p) => p.id === playlistId);
      if (playlist && !playlist.songs.some((s) => s.bvid === song.bvid)) {
        playlist.songs.push({ ...song });
      }
    }

    function removeSong(playlistId: string, bvid: string) {
      const playlist = playlists.value.find((p) => p.id === playlistId);
      if (playlist) {
        const idx = playlist.songs.findIndex((s) => s.bvid === bvid);
        if (idx >= 0) {
          playlist.songs.splice(idx, 1);
        }
      }
    }

    function moveSong(playlistId: string, from: number, to: number) {
      const playlist = playlists.value.find((p) => p.id === playlistId);
      if (!playlist || from === to) return;
      const [item] = playlist.songs.splice(from, 1);
      playlist.songs.splice(to, 0, item);
    }

    function getPlaylist(id: string): Playlist | undefined {
      return playlists.value.find((p) => p.id === id);
    }

    function setPlaylistCover(id: string, coverUrl: string) {
      const playlist = playlists.value.find((p) => p.id === id);
      if (playlist) {
        playlist.coverUrl = coverUrl;
      }
    }

    function clearPlaylistCover(id: string) {
      const playlist = playlists.value.find((p) => p.id === id);
      if (playlist) {
        playlist.coverUrl = undefined;
      }
    }

    function exportPlaylists(): string {
      const data: ExportData = {
        version: 1,
        exportedAt: new Date().toISOString(),
        playlists: playlists.value.map((p) => ({
          name: p.name,
          songs: p.songs.map((s) => ({
            bvid: s.bvid,
            title: s.title,
            author: s.author,
            duration: s.duration,
            playCount: s.playCount,
            playCountText: s.playCountText,
            coverUrl: s.coverUrl,
          })),
        })),
      };
      return JSON.stringify(data, null, 2);
    }

    function exportPlaylist(id: string): string {
      const playlist = playlists.value.find((p) => p.id === id);
      if (!playlist) return "{}";
      const data: ExportData = {
        version: 1,
        exportedAt: new Date().toISOString(),
        playlists: [
          {
            name: playlist.name,
            songs: playlist.songs.map((s) => ({
              bvid: s.bvid,
              title: s.title,
              author: s.author,
              duration: s.duration,
              playCount: s.playCount,
              playCountText: s.playCountText,
              coverUrl: s.coverUrl,
            })),
          },
        ],
      };
      return JSON.stringify(data, null, 2);
    }

    function importPlaylists(json: string): number {
      let data: ExportData;
      try {
        data = JSON.parse(json);
      } catch {
        throw new Error("无效的 JSON 文件");
      }
      if (!data.playlists || !Array.isArray(data.playlists)) {
        throw new Error("无效的歌单数据格式");
      }
      let imported = 0;
      for (const exported of data.playlists) {
        const existing = playlists.value.find((p) => p.name === exported.name);
        if (existing) {
          for (const song of exported.songs) {
            if (!existing.songs.some((s) => s.bvid === song.bvid)) {
              existing.songs.push({ ...song } as Song);
            }
          }
        } else {
          const playlist = createPlaylist(exported.name);
          playlist.songs = exported.songs.map((s) => ({ ...s } as Song));
        }
        imported++;
      }
      return imported;
    }

    return {
      playlists,
      smartPlaylists,
      createPlaylist,
      deletePlaylist,
      renamePlaylist,
      addSong,
      removeSong,
      moveSong,
      getPlaylist,
      setPlaylistCover,
      clearPlaylistCover,
      exportPlaylists,
      exportPlaylist,
      importPlaylists,
    };
  },
  {
    persist: true,
  }
);
