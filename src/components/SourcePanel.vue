<script setup lang="ts">
import { ref } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();
const urlInput = ref("");
const dragOver = ref(false);

function onDrop(e: DragEvent) {
  dragOver.value = false;
  e.preventDefault();
  const file = e.dataTransfer?.files?.[0];
  if (!file) return;
  const path = (file as File & { path?: string }).path;
  if (path) {
    store.loadLocalFile(path);
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
}
</script>

<template>
  <section class="card space-y-4">
    <h2 class="text-lg font-semibold">Source</h2>

    <div class="flex flex-col gap-2 sm:flex-row">
      <input
        v-model="urlInput"
        type="url"
        class="input-field flex-1"
        placeholder="Paste YouTube URL (https://youtube.com/watch?v=...)"
        @keydown.enter="submitYoutube"
      />
      <button class="btn-primary shrink-0" :disabled="store.loading" @click="submitYoutube">
        Load YouTube
      </button>
    </div>

    <div
      class="flex flex-col items-center justify-center rounded-xl border-2 border-dashed px-6 py-10 transition"
      :class="
        dragOver
          ? 'border-accent bg-accent/10'
          : 'border-slate-600 bg-slate-800/40 hover:border-slate-500'
      "
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
    >
      <p class="text-sm text-slate-300">Drag &amp; drop a video file here</p>
      <p class="mt-1 text-xs text-slate-500">or</p>
      <button
        class="btn-secondary mt-3"
        :disabled="store.loading"
        @click="store.loadLocalFile()"
      >
        Browse local file
      </button>
    </div>

    <div v-if="store.hasSource" class="flex items-center justify-between text-sm text-slate-400">
      <span>
        Loaded:
        <strong class="text-slate-200">
          {{ store.probe?.title ?? store.youtubeInfo?.title ?? store.localPath }}
        </strong>
        ({{ store.duration.toFixed(1) }}s)
      </span>
      <button class="text-accent hover:underline" @click="store.reset()">Clear</button>
    </div>
  </section>
</template>
