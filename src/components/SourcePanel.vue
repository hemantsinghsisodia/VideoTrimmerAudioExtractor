<script setup lang="ts">
import { ref } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();
const urlInput = ref("");
const dragOver = ref(false);
const showChangeSource = ref(false);

function onDrop(e: DragEvent) {
  dragOver.value = false;
  e.preventDefault();
  const file = e.dataTransfer?.files?.[0];
  if (!file) return;
  const path = (file as File & { path?: string }).path;
  if (path) {
    store.loadLocalFile(path);
    showChangeSource.value = false;
  } else {
    store.error = "Drag & drop requires the desktop app. Use Browse instead.";
  }
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  dragOver.value = true;
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
</script>

<template>
  <section class="card min-w-0 space-y-2 p-3">
    <div class="flex items-center justify-between gap-2">
      <h2 class="text-sm font-semibold">Source</h2>
      <button
        v-if="store.hasSource && !showChangeSource"
        type="button"
        class="text-xs text-accent hover:underline"
        @click="openChangeSource"
      >
        Change source
      </button>
    </div>

    <!-- Compact summary after load -->
    <div
      v-if="store.hasSource && !showChangeSource"
      class="flex min-w-0 items-center justify-between gap-2 rounded-lg border border-slate-700/60 bg-slate-800/50 px-2 py-1.5 text-xs"
    >
      <span class="min-w-0 truncate text-slate-300" :title="loadedTitle()">
        <strong class="text-slate-100">{{ loadedTitle() }}</strong>
        <span class="text-slate-500"> · {{ store.duration.toFixed(1) }}s</span>
      </span>
      <button
        type="button"
        class="shrink-0 text-accent hover:underline"
        @click="store.reset()"
      >
        Clear
      </button>
    </div>

    <!-- Full picker: before load or when changing source -->
    <template v-else>
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
        class="flex flex-col items-center justify-center rounded-lg border-2 border-dashed px-4 py-5 transition"
        :class="
          dragOver
            ? 'border-accent bg-accent/10'
            : 'border-slate-600 bg-slate-800/40 hover:border-slate-500'
        "
        @drop="onDrop"
        @dragover="onDragOver"
        @dragleave="onDragLeave"
      >
        <p class="text-xs text-slate-300">Drag &amp; drop a video file</p>
        <button
          class="btn-secondary mt-2 px-3 py-1.5 text-sm"
          :disabled="store.loading"
          @click="store.loadLocalFile()"
        >
          Browse file
        </button>
      </div>

      <button
        v-if="store.hasSource && showChangeSource"
        type="button"
        class="text-xs text-slate-500 hover:text-slate-300"
        @click="showChangeSource = false"
      >
        Cancel
      </button>
    </template>
  </section>
</template>
