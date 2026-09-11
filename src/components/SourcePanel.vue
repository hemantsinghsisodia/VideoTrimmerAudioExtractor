<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useMediaStore } from "@/stores/mediaStore";
import { onNativeFileDrop } from "@/services/tauri";
import { isSupportedLocalMediaPath } from "@/utils/videoFiles";
import type { UnlistenFn } from "@tauri-apps/api/event";

const store = useMediaStore();
const urlInput = ref("");
const dragOver = ref(false);
const showChangeSource = ref(false);

let unlistenFileDrop: UnlistenFn | null = null;

function handleDroppedPath(path: string) {
  if (!isSupportedLocalMediaPath(path)) {
    store.error =
      "Unsupported file type. Drop a video or MP3 audio file (mp4, mkv, webm, mov, mp3, etc.) or use Browse.";
    return;
  }
  void store.loadLocalFile(path);
  showChangeSource.value = false;
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  dragOver.value = false;
}

function onDragLeave() {
  dragOver.value = false;
}

async function submitYoutube() {
  if (!urlInput.value.trim()) return;
  await store.loadYoutube(urlInput.value.trim());
  showChangeSource.value = false;
}

function openChangeSource() {
  showChangeSource.value = true;
  urlInput.value = store.youtubeUrl || "";
}

const loadedTitle = () =>
  store.probe?.title ?? store.youtubeInfo?.title ?? store.localPath ?? "Media loaded";

onMounted(async () => {
  try {
    unlistenFileDrop = await onNativeFileDrop((event) => {
      switch (event.type) {
        case "enter":
        case "over":
          dragOver.value = true;
          break;
        case "leave":
          dragOver.value = false;
          break;
        case "drop": {
          dragOver.value = false;
          const path = event.paths[0];
          if (path) handleDroppedPath(path);
          break;
        }
      }
    });
  } catch {
    // Not running inside Tauri (e.g. Vite-only dev in browser).
  }
});

onUnmounted(() => {
  if (unlistenFileDrop) {
    void unlistenFileDrop();
    unlistenFileDrop = null;
  }
});
</script>

<template>
  <section class="card min-w-0 space-y-2 p-3">
    <div class="flex items-center justify-between gap-2">
      <h2 class="panel-title">
        <svg class="h-4 w-4 text-brand-400" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M10 13a5 5 0 0 0 7.07 0l1.41-1.41a5 5 0 0 0-7.07-7.07L10 5.93"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
          />
          <path
            d="M14 11a5 5 0 0 0-7.07 0L5.52 12.4a5 5 0 0 0 7.07 7.07L14 18.07"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
          />
        </svg>
        Source
      </h2>
      <button
        v-if="store.hasSource && !showChangeSource"
        type="button"
        class="btn-ghost"
        @click="openChangeSource"
      >
        Change source
      </button>
    </div>

    <div
      v-if="store.hasSource && !showChangeSource"
      class="flex min-w-0 items-center justify-between gap-2 rounded-xl border border-white/10 bg-white/[0.04] px-2.5 py-1.5 text-xs"
    >
      <span class="flex min-w-0 items-center gap-2 truncate text-slate-300" :title="loadedTitle()">
        <span
          class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-brand-600/20 text-brand-400"
        >
          <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M4 8.5A2.5 2.5 0 0 1 6.5 6h11A2.5 2.5 0 0 1 20 8.5v7A2.5 2.5 0 0 1 17.5 18h-11A2.5 2.5 0 0 1 4 15.5v-7Z"
              stroke="currentColor"
              stroke-width="1.6"
            />
            <path d="M10 9.8 15.5 12 10 14.2V9.8Z" fill="currentColor" />
          </svg>
        </span>
        <span class="min-w-0 truncate">
          <strong class="text-slate-100">{{ loadedTitle() }}</strong>
        </span>
      </span>
      <span class="flex shrink-0 items-center gap-2">
        <span class="rounded-full bg-white/5 px-2 py-0.5 tabular-nums text-slate-400">
          {{ store.duration.toFixed(1) }}s
        </span>
        <button type="button" class="btn-ghost" @click="store.reset()">
          Clear
        </button>
      </span>
    </div>

    <template v-else>
      <div class="space-y-3">
        <div class="flex min-w-0 flex-col gap-1.5 sm:flex-row">
          <input
            v-model="urlInput"
            type="url"
            class="input-field min-w-0 flex-1 py-1.5 text-sm"
            placeholder="YouTube URL"
            @keydown.enter="submitYoutube"
          />
          <button
            class="btn-primary shrink-0 px-3 py-1.5 text-sm"
            :disabled="store.loading"
            @click="submitYoutube"
          >
            Load
          </button>
        </div>

        <div
          class="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed px-4 py-6 transition"
          :class="
            dragOver
              ? 'border-brand-400 bg-brand-600/15 shadow-glow'
              : 'border-white/15 bg-white/[0.03] hover:border-white/25'
          "
          @drop="onDrop"
          @dragover="onDragOver"
          @dragleave="onDragLeave"
        >
        <span
          class="mb-2 flex h-10 w-10 items-center justify-center rounded-xl bg-brand-600/20 text-brand-400"
        >
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M12 16V4m0 0 4 4M12 4 8 8"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M5 16.5V18a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-1.5"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
            />
          </svg>
        </span>
        <p class="text-xs font-medium text-slate-200">Drag &amp; drop a video or MP3 audio file</p>
        <p class="mt-1 text-[10px] text-slate-500">mp4, mkv, webm, mov, avi, mp3</p>
        <button
          class="btn-secondary mt-3 px-3 py-1.5 text-sm"
          :disabled="store.loading"
          @click="store.loadLocalFile()"
        >
          Browse file
        </button>
        </div>
      </div>

      <button
        v-if="store.hasSource && showChangeSource"
        type="button"
        class="btn-ghost text-slate-500"
        @click="showChangeSource = false"
      >
        Cancel
      </button>
    </template>
  </section>
</template>
