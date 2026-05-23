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
  <div class="flex h-screen max-h-screen min-h-0 flex-col overflow-hidden p-3 gap-2">
    <header class="shrink-0 space-y-0.5">
      <h1 class="text-lg font-bold tracking-tight text-white">
        Video Trimmer &amp; Audio Extractor
      </h1>
      <p class="text-xs text-slate-400">
        Paste a YouTube URL, drag &amp; drop, or pick a local video. Trim and export.
      </p>
    </header>

    <DependencyBanner class="shrink-0" />

    <SourcePanel class="shrink-0" />

    <template v-if="store.hasSource">
      <div
        class="grid min-h-0 min-w-0 flex-1 gap-2 overflow-hidden lg:grid-cols-[minmax(0,1fr)_260px]"
      >
        <section class="card flex min-h-0 min-w-0 flex-col gap-2 overflow-hidden p-3">
          <VideoPlayer class="shrink-0" />
          <TrimTimeline class="shrink-0" />
          <TrimInputs class="shrink-0" />
        </section>
        <aside class="card flex min-h-0 min-w-0 flex-col overflow-hidden p-3">
          <ExportPanel class="min-h-0 overflow-y-auto overflow-x-hidden" />
        </aside>
      </div>
    </template>

    <div v-else class="min-h-0 flex-1" />

    <div v-if="store.error || store.lastOutputPath" class="shrink-0 space-y-1">
      <p
        v-if="store.error"
        class="line-clamp-2 break-words rounded border border-red-500/40 bg-red-950/40 px-2 py-1 text-xs text-red-300"
      >
        {{ store.error }}
      </p>
      <p
        v-if="store.lastOutputPath"
        class="line-clamp-2 break-all rounded border border-emerald-500/40 bg-emerald-950/40 px-2 py-1 text-xs text-emerald-300"
        :title="store.lastOutputPath"
      >
        Saved to: {{ store.lastOutputPath }}
        <span v-if="store.downloadedPath" class="block text-emerald-400/80">
          Playing from app cache.
        </span>
      </p>
    </div>

    <div
      v-if="store.exporting || store.loading"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    >
      <div class="card max-w-sm space-y-2 p-4 text-center">
        <p class="text-sm font-medium text-white">
          {{ store.exporting ? "Export in progress…" : "Loading…" }}
        </p>
        <p class="text-xs text-slate-400">This may take a minute for long videos.</p>
        <div v-if="store.progress" class="space-y-1">
          <div class="h-1.5 overflow-hidden rounded-full bg-slate-700">
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
