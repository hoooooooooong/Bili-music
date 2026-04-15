<script setup lang="ts">
import { ref, reactive, watch, nextTick } from "vue";
import { NIcon, NSpin } from "naive-ui";
import { CloseOutline, ChatbubblesOutline, ChevronDownOutline, ChevronUpOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "@/stores/player";
import type { Comment, CommentResponse } from "@/types";

const emit = defineEmits<{ close: [] }>();
const player = usePlayerStore();

const comments = ref<Comment[]>([]);
const page = ref(1);
const isEnd = ref(false);
const loading = ref(false);
const loadingMore = ref(false);
const error = ref("");
const isLimited = ref(false);

// Reply expansion state
interface ReplyState {
  replies: Comment[];
  loading: boolean;
  loadingMore: boolean;
  isEnd: boolean;
  page: number;
  expanded: boolean;
}
const replyStates = reactive<Record<number, ReplyState>>({});

async function fetchComments(reset = false) {
  if (!player.currentSong) return;
  if (loading.value || loadingMore.value) return;
  if (!reset && isEnd.value) return;

  if (reset) {
    loading.value = true;
    error.value = "";
    isLimited.value = false;
  } else {
    loadingMore.value = true;
  }

  try {
    const res = await invoke<CommentResponse>("get_video_comments", {
      bvid: player.currentSong.bvid,
      page: reset ? 1 : page.value,
    });

    if (reset) {
      comments.value = res.comments;
      page.value = 1;
      // Clear reply states
      for (const key of Object.keys(replyStates)) {
        delete replyStates[key];
      }
    } else {
      comments.value.push(...res.comments);
    }
    isEnd.value = res.isEnd;
    // B站对未登录用户限制返回 3 条，提示用户登录
    if (reset && res.comments.length === 3 && res.isEnd) {
      isLimited.value = true;
    } else {
      isLimited.value = false;
    }
    page.value++;
  } catch (e: any) {
    if (reset) {
      error.value = typeof e === "string" ? e : "获取评论失败";
    }
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

async function toggleReplies(c: Comment) {
  const state = replyStates[c.rpid];
  if (!state) {
    // First time: create state and fetch
    replyStates[c.rpid] = {
      replies: [],
      loading: false,
      loadingMore: false,
      isEnd: false,
      page: 1,
      expanded: true,
    };
    await fetchReplies(c);
  } else if (state.expanded) {
    state.expanded = false;
  } else {
    state.expanded = true;
    if (state.replies.length === 0) {
      await fetchReplies(c);
    }
  }
}

async function fetchReplies(c: Comment) {
  const state = replyStates[c.rpid];
  if (!state || state.loading || state.loadingMore || state.isEnd) return;
  if (!player.currentSong) return;

  state.loading = true;
  try {
    const res = await invoke<CommentResponse>("get_comment_replies", {
      bvid: player.currentSong.bvid,
      root: c.rpid,
      page: state.page,
    });
    state.replies.push(...res.comments);
    state.isEnd = res.isEnd;
    state.page++;
  } catch {
    // silently fail
  } finally {
    state.loading = false;
  }
}

async function loadMoreReplies(c: Comment) {
  const state = replyStates[c.rpid];
  if (!state || state.loading || state.loadingMore || state.isEnd) return;

  state.loadingMore = true;
  try {
    const res = await invoke<CommentResponse>("get_comment_replies", {
      bvid: player.currentSong!.bvid,
      root: c.rpid,
      page: state.page,
    });
    state.replies.push(...res.comments);
    state.isEnd = res.isEnd;
    state.page++;
  } catch {
    // silently fail
  } finally {
    state.loadingMore = false;
  }
}

function handleScroll(e: Event) {
  const el = e.target as HTMLElement;
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 50) {
    fetchComments();
  }
}

function formatTime(ctime: number): string {
  if (!ctime) return "";
  const d = new Date(ctime * 1000);
  const now = new Date();
  const diff = Math.floor((now.getTime() - d.getTime()) / 1000);

  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 2592000) return `${Math.floor(diff / 86400)} 天前`;

  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function formatCount(n: number): string {
  if (n >= 10000) return (n / 10000).toFixed(1) + "万";
  return n.toString();
}

const levelColors: Record<number, string> = {
  0: "#a0a0a0",
  1: "#a0a0a0",
  2: "#8db7c7",
  3: "#62a5e0",
  4: "#f6b740",
  5: "#f26d5e",
  6: "#f26d5e",
};

watch(
  () => player.currentSong?.bvid,
  async () => {
    comments.value = [];
    page.value = 1;
    isEnd.value = false;
    isLimited.value = false;
    error.value = "";
    for (const key of Object.keys(replyStates)) {
      delete replyStates[key];
    }
    await nextTick();
    fetchComments(true);
  }
);

// Initial fetch when panel opens
watch(
  () => player.currentSong?.bvid,
  (bvid) => {
    if (bvid && comments.value.length === 0 && !loading.value) {
      fetchComments(true);
    }
  },
  { immediate: true }
);
</script>

<template>
  <div class="comment-panel">
    <div class="comment-header">
      <h3>
        <NIcon size="16" style="margin-right: 6px; vertical-align: -2px">
          <ChatbubblesOutline />
        </NIcon>
        评论
      </h3>
      <button class="comment-close-btn" @click="emit('close')">
        <NIcon size="14"><CloseOutline /></NIcon>
      </button>
    </div>

    <div v-if="loading" class="comment-loading">
      <NSpin :size="24" />
      <p>加载评论中...</p>
    </div>

    <div v-else-if="error" class="comment-error">
      <p>{{ error }}</p>
      <button class="comment-retry" @click="fetchComments(true)">重试</button>
    </div>

    <div
      v-else
      class="comment-list"
      @scroll="handleScroll"
    >
      <div v-if="comments.length === 0" class="comment-empty">
        <NIcon size="32" style="color: rgba(255,255,255,0.15)">
          <ChatbubblesOutline />
        </NIcon>
        <p>暂无评论</p>
      </div>

      <div v-for="c in comments" :key="c.rpid" class="comment-item">
        <img
          class="comment-avatar"
          :src="c.member.avatar"
          :alt="c.member.name"
          loading="lazy"
        />
        <div class="comment-body">
          <div class="comment-meta">
            <span class="comment-name">{{ c.member.name }}</span>
            <span
              class="comment-level"
              :style="{ color: levelColors[c.member.level] || '#a0a0a0' }"
            >LV{{ c.member.level }}</span>
            <span class="comment-time">{{ formatTime(c.ctime) }}</span>
          </div>
          <p class="comment-text">{{ c.message }}</p>
          <div class="comment-actions">
            <span class="comment-like">
              <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
              </svg>
              {{ formatCount(c.like) }}
            </span>
            <button
              v-if="c.rcount > 0"
              class="comment-rcount-btn"
              @click="toggleReplies(c)"
            >
              回复 {{ formatCount(c.rcount) }}
              <NIcon size="10">
                <ChevronDownOutline v-if="!replyStates[c.rpid]?.expanded" />
                <ChevronUpOutline v-else />
              </NIcon>
            </button>
          </div>

          <!-- Replies section -->
          <div v-if="replyStates[c.rpid]?.expanded" class="comment-replies">
            <div v-if="replyStates[c.rpid].loading" class="replies-loading">
              <NSpin :size="14" />
            </div>
            <template v-else>
              <div v-for="r in replyStates[c.rpid].replies" :key="r.rpid" class="reply-item">
                <img
                  class="reply-avatar"
                  :src="r.member.avatar"
                  :alt="r.member.name"
                  loading="lazy"
                />
                <div class="reply-body">
                  <div class="reply-meta">
                    <span class="reply-name">{{ r.member.name }}</span>
                    <span
                      class="comment-level"
                      :style="{ color: levelColors[r.member.level] || '#a0a0a0' }"
                    >LV{{ r.member.level }}</span>
                    <span class="comment-time">{{ formatTime(r.ctime) }}</span>
                  </div>
                  <p class="reply-text">{{ r.message }}</p>
                  <div class="reply-actions">
                    <span class="comment-like">
                      <svg viewBox="0 0 24 24" width="11" height="11" fill="currentColor">
                        <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                      </svg>
                      {{ formatCount(r.like) }}
                    </span>
                  </div>
                </div>
              </div>
              <button
                v-if="!replyStates[c.rpid].isEnd && replyStates[c.rpid].replies.length > 0"
                class="replies-more-btn"
                :disabled="replyStates[c.rpid].loadingMore"
                @click="loadMoreReplies(c)"
              >
                <NSpin v-if="replyStates[c.rpid].loadingMore" :size="12" />
                <template v-else>展开更多回复</template>
              </button>
            </template>
          </div>
        </div>
      </div>

      <div v-if="loadingMore" class="comment-loading-more">
        <NSpin :size="18" />
      </div>
      <div v-else-if="isLimited" class="comment-end">
        仅展示部分评论，登录后可查看更多
      </div>
      <div v-else-if="isEnd && comments.length > 0" class="comment-end">
        没有更多了
      </div>
    </div>
  </div>
</template>

<style scoped>
.comment-panel {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 360px;
  z-index: 10;
  background: rgba(20, 20, 30, 0.95);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.4);
}

.comment-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.comment-header h3 {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin: 0;
}

.comment-close-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.5);
  border-radius: 6px;
  cursor: pointer;
  background: none;
  border: none;
}

.comment-close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.comment-loading,
.comment-error,
.comment-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  color: rgba(255, 255, 255, 0.3);
  font-size: 13px;
}

.comment-retry {
  font-size: 12px;
  color: var(--accent-color);
  background: rgba(255, 255, 255, 0.08);
  border: none;
  padding: 6px 16px;
  border-radius: 16px;
  cursor: pointer;
}

.comment-retry:hover {
  background: rgba(255, 255, 255, 0.15);
}

.comment-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.comment-list::-webkit-scrollbar {
  width: 4px;
}

.comment-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.comment-item {
  display: flex;
  gap: 10px;
  padding: 12px 16px;
}

.comment-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.comment-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  flex-shrink: 0;
  object-fit: cover;
}

.comment-body {
  flex: 1;
  min-width: 0;
}

.comment-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.comment-name {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  max-width: 100px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.comment-level {
  font-size: 10px;
  font-weight: 600;
  flex-shrink: 0;
}

.comment-time {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.25);
  margin-left: auto;
  flex-shrink: 0;
}

.comment-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.5;
  margin: 0 0 4px;
  word-break: break-word;
}

.comment-actions {
  display: flex;
  gap: 12px;
  margin-top: 4px;
}

.comment-like {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
}

.comment-rcount-btn {
  display: flex;
  align-items: center;
  gap: 2px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}

.comment-rcount-btn:hover {
  color: rgba(255, 255, 255, 0.6);
}

/* Replies */
.comment-replies {
  margin-top: 8px;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
}

.replies-loading {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.reply-item {
  display: flex;
  gap: 8px;
  padding: 8px 0;
}

.reply-item + .reply-item {
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.reply-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  flex-shrink: 0;
  object-fit: cover;
}

.reply-body {
  flex: 1;
  min-width: 0;
}

.reply-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 2px;
}

.reply-name {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.55);
  max-width: 90px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.reply-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.5;
  margin: 0 0 2px;
  word-break: break-word;
}

.reply-actions {
  display: flex;
  gap: 10px;
  margin-top: 2px;
}

.replies-more-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 6px 0 2px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  background: none;
  border: none;
  cursor: pointer;
}

.replies-more-btn:hover:not(:disabled) {
  color: rgba(255, 255, 255, 0.55);
}

.replies-more-btn:disabled {
  cursor: not-allowed;
}

.comment-loading-more {
  display: flex;
  justify-content: center;
  padding: 12px;
}

.comment-end {
  text-align: center;
  padding: 16px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.2);
}
</style>
