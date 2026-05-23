# Video Trimmer & Audio Extractor

Desktop app built with **Tauri 2**, **Vue 3**, **Vite**, **TypeScript**, **Pinia**, and **Tailwind CSS**. Trims local videos, extracts audio, and downloads YouTube content with format/quality selection via **FFmpeg** and **yt-dlp**.

## Features

- Paste a YouTube URL, drag & drop, or browse for a local video
- Video preview (local files) and timeline trim handles
- Start/end time text inputs with validation
- **Local:** export trimmed video or audio-only (best quality AAC/MP3)
- **YouTube:** list all available formats/qualities and download with trim sections

## Prerequisites

1. [Node.js](https://nodejs.org/) 18+
2. [Rust](https://www.rust-lang.org/tools/install) (for Tauri)
3. [FFmpeg](https://ffmpeg.org/) on PATH (`ffmpeg`, `ffprobe`)
4. [yt-dlp](https://github.com/yt-dlp/yt-dlp) on PATH

Windows quick install:

```powershell
winget install Gyan.FFmpeg
pip install yt-dlp
```

Install Rust:

```powershell
winget install Rustlang.Rustup
```

## Setup

```powershell
cd VideoTrimmerAudioExtractor
npm install
```

Generate app icons (optional, required for release builds):

```powershell
npm run tauri icon path\to\your-icon.png
```

## Development

```powershell
npm run tauri dev
```

## Build

```powershell
npm run tauri build
```

## Tests

```powershell
npm test
```

## Project structure

```
src/                 Vue frontend (components, stores, utils)
src-tauri/src/       Rust backend (FFmpeg, yt-dlp commands)
```

## Notes

- YouTube preview shows thumbnail only; full playback is after download.
- Drag & drop of local files works in the Tauri desktop app (uses native file paths).
- Ensure FFmpeg and yt-dlp are on your system PATH before exporting.
