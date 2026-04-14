# Bili Music App - Project Context

Desktop application for searching, streaming, and downloading music from Bilibili, built with Tauri v2 and Vue 3.

## Tech Stack
- **Frontend**: Vue 3.5 (Composition API, `<script setup>`), TypeScript 5.7, Vite 6.
- **UI Framework**: Naive UI, @vicons/ionicons5.
- **State Management**: Pinia with `pinia-plugin-persistedstate`.
- **Backend**: Rust (Tauri 2), Tokio (Async runtime), Reqwest (HTTP client).
- **Core Tools**: FFmpeg (used for converting Bilibili M4A streams to MP3/AAC/FLAC/WAV).
- **Packaging**: NSIS for Windows installers.

## Project Structure

### Frontend (`src/`)
- `components/`: Organized by feature (player, search, playlists, favorites, settings).
- `composables/`: Reusable logic for audio control, lyrics, media session, window management, etc.
- `stores/`: Pinia stores for application state (player, search, download, history, etc.).
- `pages/`: Top-level views (Home, Download).
- `utils/`: Helpers for LRC parsing, formatting, and event emitting.
- `styles/`: Global and theme-based (CSS variables) styling.

### Backend (`src-tauri/`)
- `src/lib.rs`: Main application logic, plugin initialization, system tray setup, and protocol registration.
- `src/commands/`: Tauri commands exposed to the frontend (search, download, lyrics, settings).
- `src/core/`: Business logic.
    - `searcher.rs`: Bilibili API interaction for searching and metadata.
    - `downloader.rs`: Download management and Bilibili client initialization.
    - `converter.rs`: FFmpeg-based audio conversion logic.
    - `task_manager.rs`: Progress tracking for downloads.
    - `ffmpeg_path.rs`: Logic for locating the FFmpeg binary.
- `binaries/`: Local storage for `ffmpeg.exe` (downloaded at build time).

## Key Commands
- `pnpm dev`: Start Vite development server.
- `pnpm tauri dev`: Start Tauri development mode (Frontend + Backend).
- `pnpm build`: Build the frontend assets.
- `pnpm tauri build`: Build the production installer (NSIS).
- `pnpm download-ffmpeg`: Download the required FFmpeg binary for conversion.

## Development Conventions

### Frontend
- **Composition API**: Use `<script setup lang="ts">` for all Vue components.
- **Components**: Group components by functional domain in `src/components/`.
- **Styling**: Prefer CSS variables and Naive UI's theme overrides. Custom window decorations require `src/components/TitleBar.vue`.
- **State**: Use Pinia for all shared state. Persist user preferences and library data using the persistence plugin.

### Backend (Rust)
- **Tauri Commands**: Keep command handlers in `src/commands/` and delegate heavy logic to `src/core/`.
- **Error Handling**: Use the custom `Error` type defined in `src/error.rs` for consistent error reporting to the frontend.
- **Async**: Use `tauri::async_runtime` for background tasks (e.g., downloads, image fetching).
- **State Management**: Use `app.manage()` in `setup` to provide global resources (Searcher, TaskManager, etc.) to command handlers.

## Architecture Highlights
- **Custom Protocol**: `bili-cover://` is registered to bypass Bilibili's referer checks when fetching cover images.
- **Multi-Window**: Supports a main window, a mini player, and a desktop lyrics window.
- **FFmpeg Integration**: The app expects `ffmpeg.exe` in `src-tauri/binaries/` or in the system PATH. It is bundled as a resource in production.
- **Media Session**: Integrates with the system media controller (play/pause/prev/next/metadata) via `useMediaSession.ts`.
- **System Tray**: Provides playback controls and window visibility management from the OS tray.

## Constraints & Gotchas
- **FFmpeg**: Must be the GPL version to include `libmp3lame` for MP3 encoding.
- **Bilibili API**: Requires proper `User-Agent` and `Referer` headers for most requests.
- **Window Decorations**: `decorations: false` is used for a custom look; dragging and window controls are implemented in the frontend.
