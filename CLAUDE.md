# Bili Music App

B站音乐搜索与下载桌面应用，基于 Tauri v2 + Vue 3 构建。

## 技术栈

- **前端**: Vue 3.5 (Composition API + `<script setup>`) / TypeScript 5.7 / Vite 6
- **UI 框架**: Naive UI / @vicons/ionicons5
- **状态管理**: Pinia + pinia-plugin-persistedstate
- **路由**: Vue Router 4 (History 模式)
- **后端**: Rust (Tauri 2) / tokio / reqwest / serde
- **包管理器**: pnpm
- **打包**: NSIS 安装包 (Windows)

## 项目结构

```
src/                        # 前端源码
├── components/             # 按功能分组的 Vue 组件
│   ├── player/             # 播放器相关 (FullPlayer, MiniPlayer, PlaylistPanel 等)
│   ├── search/             # 搜索相关 (SearchBar, SongCard, ResultGrid 等)
│   ├── playlists/          # 歌单管理
│   ├── favorites/          # 收藏列表
│   └── settings/           # 设置页面
├── composables/            # Vue 组合式函数
│   ├── useAudio.ts         # 音频上下文
│   ├── useLyrics.ts        # 歌词处理
│   ├── usePlayerControls.ts
│   ├── useMediaSession.ts  # 系统媒体控制
│   ├── useKeyboardShortcuts.ts
│   ├── useDragSort.ts
│   └── useAudioCache.ts
├── stores/                 # Pinia 状态仓库
├── pages/                  # 页面组件 (HomePage, DownloadPage, SettingsPage)
├── router/index.ts         # 路由定义
├── styles/theme.css        # 主题 CSS 变量 (明/暗模式)
├── types/index.ts          # TypeScript 类型定义
└── utils/                  # 工具函数 (lrc-parser, formatters, emitter)

src-tauri/                  # Tauri 后端
├── src/
│   ├── main.rs             # Tauri 应用入口，注册插件
│   ├── lib.rs              # 库初始化，系统托盘
│   ├── commands/           # Tauri 命令 (search, download, lyrics, settings)
│   └── core/               # 核心业务逻辑
│       ├── searcher.rs     # B站搜索
│       ├── downloader.rs   # 下载管理
│       ├── converter.rs    # FFmpeg 音频转换 (MP3/AAC/FLAC/WAV)
│       ├── ffmpeg_path.rs  # FFmpeg 路径解析
│       ├── lyrics_client.rs
│       └── task_manager.rs # 任务管理
├── capabilities/default.json
├── tauri.conf.json
└── binaries/ffmpeg.exe     # FFmpeg 二进制 (git 忽略，构建时自动下载)
```

## 常用命令

```bash
pnpm dev                    # 启动前端开发服务器
pnpm tauri dev              # 启动 Tauri 开发模式
pnpm build                  # 构建前端
pnpm tauri build            # 构建完整应用 (自动下载 ffmpeg)
pnpm download-ffmpeg        # 手动下载 ffmpeg
pnpm download-ffmpeg -- --Force  # 强制重新下载
```

## 开发规范

- 所有 Vue 组件使用 `<script setup lang="ts">`，统一使用 Composition API
- 组件按功能域组织在 `components/` 子目录中
- 路径别名 `@/` 映射到 `src/`
- 主题通过 CSS 变量 + `data-theme` 属性切换明暗模式
- Rust 端命令通过 `#[tauri::command]` 宏暴露给前端调用
- FFmpeg 用于音频格式转换 (B站 M4A → MP3/AAC/FLAC/WAV)，需 GPL 版本 (含 libmp3lame)
- 窗口使用自定义标题栏 (decorations: false)

## Tauri 插件

dialog (文件对话框) / fs (文件系统) / notification (通知) / autostart (开机自启) / global-shortcut (全局快捷键) / shell (打开外部链接)

## 注意事项

- `src-tauri/binaries/ffmpeg.exe` 被 git 忽略，首次构建或 `pnpm download-ffmpeg` 时自动下载
- `src-tauri/target/` 和 `dist/` 为构建产物，已忽略
- TypeScript 开启了严格模式
