<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NIcon, NDropdown, NModal, NInput, NButton, useMessage } from "naive-ui";
import SearchBar from "./search/SearchBar.vue";
import { HomeOutline, DownloadOutline, LogOutOutline, PersonOutline } from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/stores/settings";

const router = useRouter();
const route = useRoute();
const settingsStore = useSettingsStore();
const message = useMessage();

const emit = defineEmits<{
  "login-changed": [];
}>();

const activeTab = computed(() => {
  if (route.path === "/downloads") return "downloads";
  return "home";
});

function switchTab(tab: string) {
  if (tab === "home") router.push("/");
  else if (tab === "downloads") router.push("/downloads");
}

const isLoggedIn = ref(false);
const userInfo = ref<{ mid: number; uname: string; face: string } | null>(null);
const loginVersion = ref(0);

const showLoginModal = ref(false);
const loginInput = ref("");
const loginLoading = ref(false);

const userDropdownOptions = [
  { label: "退出登录", key: "logout" },
];

async function fetchUserInfo() {
  try {
    isLoggedIn.value = await invoke<boolean>("check_login");
    if (isLoggedIn.value) {
      userInfo.value = await invoke("get_user_info");
    } else {
      userInfo.value = null;
    }
  } catch {
    isLoggedIn.value = false;
    userInfo.value = null;
  }
}

onMounted(fetchUserInfo);

async function handleUserDropdownSelect(key: string) {
  if (key === "logout") {
    settingsStore.sessdata = "";
    await settingsStore.saveSettings();
    isLoggedIn.value = false;
    userInfo.value = null;
    emit("login-changed");
  }
}

async function handleLogin() {
  const val = loginInput.value.trim();
  if (!val) {
    message.warning("请输入 SESSDATA");
    return;
  }
  loginLoading.value = true;
  try {
    settingsStore.sessdata = val;
    await settingsStore.saveSettings();
    // Wait a moment for cookie jar to update
    await new Promise((r) => setTimeout(r, 500));
    await fetchUserInfo();
    showLoginModal.value = false;
    loginInput.value = "";
    if (isLoggedIn.value) {
      message.success("登录成功");
      emit("login-changed");
    } else {
      message.error("登录失败，请检查 SESSDATA 是否正确");
    }
  } catch {
    message.error("保存失败");
  } finally {
    loginLoading.value = false;
  }
}

function openLoginModal() {
  loginInput.value = "";
  showLoginModal.value = true;
}
</script>

<template>
  <div class="app-header">
    <div class="header-left">
      <button
        class="nav-btn"
        :class="{ active: activeTab === 'home' }"
        @click="switchTab('home')"
      >
        <NIcon size="18"><HomeOutline /></NIcon>
        <span>发现</span>
      </button>
      <button
        class="nav-btn"
        :class="{ active: activeTab === 'downloads' }"
        @click="switchTab('downloads')"
      >
        <NIcon size="18"><DownloadOutline /></NIcon>
        <span>下载</span>
      </button>
    </div>
    <SearchBar />
    <div class="header-right">
      <template v-if="isLoggedIn && userInfo">
        <NDropdown
          :options="userDropdownOptions"
          @select="handleUserDropdownSelect"
          trigger="click"
          placement="bottom-end"
        >
          <button class="user-btn">
            <img class="user-avatar" :src="userInfo.face" alt="" />
            <span class="user-name">{{ userInfo.uname }}</span>
          </button>
        </NDropdown>
      </template>
      <template v-else>
        <button class="not-logged-in" @click="openLoginModal">
          <NIcon size="16"><PersonOutline /></NIcon>
          未登录
        </button>
      </template>
    </div>
  </div>

  <NModal
    v-model:show="showLoginModal"
    preset="dialog"
    title="登录B站账号"
    positive-text="确认"
    negative-text="取消"
    :loading="loginLoading"
    @positive-click="handleLogin"
  >
    <div class="login-modal-body">
      <p class="login-hint">
        请在浏览器登录 B站 后，从 Cookie 中复制 SESSDATA 的值粘贴到下方。
      </p>
      <NInput
        v-model:value="loginInput"
        type="textarea"
        placeholder="粘贴你的 SESSDATA"
        :autosize="{ minRows: 2, maxRows: 4 }"
        @keydown.enter.ctrl="handleLogin"
      />
    </div>
  </NModal>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  gap: 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--header-bg);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.header-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  margin-left: auto;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.nav-btn:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.nav-btn.active {
  background: var(--accent-light);
  color: var(--accent-color);
  font-weight: 500;
}

.user-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px 4px 4px;
  border-radius: 20px;
  cursor: pointer;
  border: none;
  background: transparent;
  transition: background 0.15s;
}

.user-btn:hover {
  background: var(--card-hover);
}

.user-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  object-fit: cover;
}

.user-name {
  font-size: 13px;
  color: var(--app-text);
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.not-logged-in {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  border: none;
  background: transparent;
  padding: 6px 12px;
  border-radius: 8px;
  transition: all 0.15s;
}

.not-logged-in:hover {
  background: var(--card-hover);
  color: var(--app-text);
}

.login-modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.login-hint {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
