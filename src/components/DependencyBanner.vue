<script setup lang="ts">
import { computed } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();

const missing = computed(() => {
  if (!store.deps) return [];
  const list: string[] = [];
  if (!store.deps.ffmpeg) list.push("ffmpeg");
  if (!store.deps.ffprobe) list.push("ffprobe");
  if (!store.deps.ytdlp) list.push("yt-dlp");
  return list;
});
</script>

<template>
  <div
    v-if="missing.length"
    class="rounded border border-amber-500/40 bg-amber-950/30 px-2 py-1.5 text-xs text-amber-200"
  >
    <p class="font-medium">Missing: {{ missing.join(", ") }}</p>
    <p class="mt-1 text-[10px] text-amber-300/80">
      Install FFmpeg and yt-dlp, then restart the app. On Windows:
      <code class="rounded bg-slate-800 px-1">winget install Gyan.FFmpeg</code> and
      <code class="rounded bg-slate-800 px-1">pip install yt-dlp</code>
    </p>
  </div>
</template>
