<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useMediaStore } from "@/stores/mediaStore";
import SourcePanel from "@/components/SourcePanel.vue";
import VideoPlayer from "@/components/VideoPlayer.vue";
import TrimTimeline from "@/components/TrimTimeline.vue";
import TrimInputs from "@/components/TrimInputs.vue";
import ExportPanel from "@/components/ExportPanel.vue";
import DependencyBanner from "@/components/DependencyBanner.vue";
import { revealInFolder } from "@/services/tauri";

const store = useMediaStore();

const progressPercent = computed(() => Math.round(store.progress?.percent ?? 0));
const ringOffset = computed(() => {
  const circumference = 2 * Math.PI * 42;
  const pct = Math.min(100, Math.max(0, store.progress?.percent ?? 0));
  return circumference * (1 - pct / 100);
});

onMounted(() => {
  store.init();
});

async function showInFolder() {
  if (!store.lastOutputPath) return;
  try {
    await revealInFolder(store.lastOutputPath);
  } catch (e) {
    store.error = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <div class="relative flex h-screen max-h-screen min-h-0 flex-col overflow-hidden">
    <div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden bg-[#060814]">
      <div
        class="absolute -left-24 -top-28 h-80 w-80 rounded-full bg-brand-600/30 blur-3xl animate-ambient"
      />
      <div
        class="absolute -bottom-24 -right-16 h-96 w-96 rounded-full bg-violet-500/20 blur-3xl animate-ambient"
      />
      <div
        class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgba(99,102,241,0.18),transparent_55%)]"
      />
    </div>

    <div class="flex h-screen max-h-screen min-h-0 flex-col overflow-hidden p-3 gap-2">
      <header class="flex shrink-0 items-center gap-3">
        <div
          class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-brand-600 to-brand-400 shadow-glow"
        >
          <svg class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M4 7.5A2.5 2.5 0 0 1 6.5 5h11A2.5 2.5 0 0 1 20 7.5v9A2.5 2.5 0 0 1 17.5 19h-11A2.5 2.5 0 0 1 4 16.5v-9Z"
              stroke="currentColor"
              stroke-width="1.6"
            />
            <path d="M10 9.5 16 12l-6 2.5v-5Z" fill="currentColor" />
          </svg>
        </div>
        <div class="min-w-0">
          <h1
            class="bg-gradient-to-r from-white via-slate-100 to-brand-400 bg-clip-text text-lg font-bold tracking-tight text-transparent"
          >
            Video Trimmer &amp; Audio Extractor
          </h1>
          <p class="text-xs text-slate-400">
            Paste a YouTube URL, drag &amp; drop, or pick a local video. Trim and export.
          </p>
        </div>
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
            <ExportPanel class="thin-scroll min-h-0 overflow-y-auto overflow-x-hidden" />
          </aside>
        </div>
      </template>

      <div v-else class="min-h-0 flex-1" />

      <div v-if="store.error || store.lastOutputPath" class="shrink-0 space-y-1">
        <p
          v-if="store.error"
          class="status-pill line-clamp-2 break-words border-red-400/30 bg-red-950/50 text-red-200"
        >
          <svg class="mt-0.5 h-3.5 w-3.5 shrink-0" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M12 9v4m0 4h.01M10.3 4.7 2.8 18a2 2 0 0 0 1.7 3h15a2 2 0 0 0 1.7-3L13.7 4.7a2 2 0 0 0-3.4 0Z"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span>{{ store.error }}</span>
        </p>
        <p
          v-if="store.lastOutputPath"
          class="status-pill break-all border-emerald-400/30 bg-emerald-950/40 text-emerald-200"
          :title="store.lastOutputPath"
        >
          <svg class="mt-0.5 h-3.5 w-3.5 shrink-0" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M20 7 10 17l-6-6"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span class="min-w-0 flex-1">
            Saved to: {{ store.lastOutputPath }}
            <span v-if="store.downloadedPath" class="block text-emerald-300/80">
              Playing from app cache.
            </span>
          </span>
          <button
            type="button"
            class="btn-ghost shrink-0 whitespace-nowrap text-emerald-100"
            @click="showInFolder"
          >
            Show in folder
          </button>
        </p>
      </div>
    </div>

    <div
      v-if="store.exporting || store.loading"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/55 backdrop-blur-md"
    >
      <div class="card w-full max-w-sm space-y-4 p-5 text-center">
        <p class="text-sm font-medium text-white">
          {{ store.exporting ? "Working…" : "Loading…" }}
        </p>
        <div class="relative mx-auto h-28 w-28">
          <svg class="h-28 w-28 -rotate-90" viewBox="0 0 100 100" aria-hidden="true">
            <circle
              cx="50"
              cy="50"
              r="42"
              fill="none"
              stroke="rgba(148,163,184,0.18)"
              stroke-width="8"
            />
            <circle
              cx="50"
              cy="50"
              r="42"
              fill="none"
              stroke="url(#progressGlow)"
              stroke-width="8"
              stroke-linecap="round"
              :stroke-dasharray="2 * Math.PI * 42"
              :stroke-dashoffset="ringOffset"
              class="transition-[stroke-dashoffset] duration-200 ease-out"
            />
            <defs>
              <linearGradient id="progressGlow" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stop-color="#6366f1" />
                <stop offset="100%" stop-color="#a78bfa" />
              </linearGradient>
            </defs>
          </svg>
          <p
            class="absolute inset-0 flex items-center justify-center text-2xl font-bold tabular-nums text-white"
          >
            {{ progressPercent }}%
          </p>
        </div>
        <p v-if="store.progress" class="text-xs text-slate-300">
          {{ store.progress.message }}
        </p>
        <p v-else class="text-xs text-slate-400">Please wait…</p>
        <button
          type="button"
          class="btn-secondary w-full py-1.5 text-sm"
          :disabled="store.cancelling"
          @click="store.cancelCurrentJob()"
        >
          {{ store.cancelling ? "Cancelling…" : "Cancel" }}
        </button>
      </div>
    </div>
  </div>
</template>
