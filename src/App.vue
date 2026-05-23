<script setup lang="ts">
import { onMounted } from "vue";
import { useMediaStore } from "@/stores/mediaStore";
import SourcePanel from "@/components/SourcePanel.vue";
import VideoPlayer from "@/components/VideoPlayer.vue";
import TrimTimeline from "@/components/TrimTimeline.vue";
import TrimInputs from "@/components/TrimInputs.vue";
import ExportPanel from "@/components/ExportPanel.vue";
import DependencyBanner from "@/components/DependencyBanner.vue";

const store = useMediaStore();

onMounted(() => {
  store.init();
});
</script>

<template>
  <div class="mx-auto flex min-h-screen max-w-6xl flex-col gap-6 p-6">
    <header class="space-y-1">
      <h1 class="text-2xl font-bold tracking-tight text-white">
        Video Trimmer &amp; Audio Extractor
      </h1>
      <p class="text-sm text-slate-400">
        Paste a YouTube URL, drag &amp; drop, or pick a local video. Trim on the timeline and export.
      </p>
    </header>

    <DependencyBanner />

    <SourcePanel />

    <template v-if="store.hasSource">
      <div class="grid min-w-0 gap-6 lg:grid-cols-[minmax(0,1fr)_320px]">
        <section class="card min-w-0 space-y-4">
          <VideoPlayer />
          <TrimTimeline />
          <TrimInputs />
        </section>
        <aside class="card">
          <ExportPanel />
        </aside>
      </div>
    </template>

    <p
      v-if="store.error"
      class="break-words rounded-lg border border-red-500/40 bg-red-950/40 px-4 py-2 text-sm text-red-300"
    >
      {{ store.error }}
    </p>

    <p
      v-if="store.lastOutputPath"
      class="break-all rounded-lg border border-emerald-500/40 bg-emerald-950/40 px-4 py-2 text-sm text-emerald-300"
    >
      Saved to: {{ store.lastOutputPath }}
      <span v-if="store.downloadedPath" class="block text-xs text-emerald-400/80">
        Playing from app cache (staged copy for preview).
      </span>
    </p>

    <div
      v-if="store.exporting || store.loading"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    >
      <div class="card max-w-sm space-y-3 text-center">
        <p class="font-medium text-white">
          {{ store.exporting ? "Export in progress…" : "Loading…" }}
        </p>
        <p class="text-sm text-slate-400">
          The app stays responsive; this may take a minute for long videos.
        </p>
        <div v-if="store.progress" class="space-y-1">
          <div class="h-2 overflow-hidden rounded-full bg-slate-700">
            <div
              class="h-full bg-accent transition-all"
              :style="{ width: `${store.progress.percent}%` }"
            />
          </div>
          <p class="text-xs text-slate-400">{{ store.progress.message }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
