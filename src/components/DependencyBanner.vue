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

const outdated = computed(() => store.deps?.ytdlp_outdated === true);
const outdatedVersion = computed(() => store.deps?.ytdlp_version ?? "unknown");
</script>

<template>
  <div v-if="missing.length || outdated" class="space-y-2">
    <div
      v-if="missing.length"
      class="status-pill border-amber-400/30 bg-amber-950/40 text-amber-100"
    >
      <svg class="mt-0.5 h-4 w-4 shrink-0 text-amber-300" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M12 9v4m0 4h.01M10.3 4.7 2.8 18a2 2 0 0 0 1.7 3h15a2 2 0 0 0 1.7-3L13.7 4.7a2 2 0 0 0-3.4 0Z"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <div>
        <p class="font-medium">Missing: {{ missing.join(", ") }}</p>
        <p class="mt-1 text-[10px] text-amber-200/80">
          Install FFmpeg and yt-dlp, then restart the app. On Windows:
          <code class="rounded bg-black/30 px-1">winget install Gyan.FFmpeg</code> and
          <code class="rounded bg-black/30 px-1">winget install yt-dlp.yt-dlp</code>
        </p>
      </div>
    </div>
    <div
      v-if="outdated"
      class="status-pill border-amber-400/30 bg-amber-950/40 text-amber-100"
    >
      <svg class="mt-0.5 h-4 w-4 shrink-0 text-amber-300" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M12 9v4m0 4h.01M10.3 4.7 2.8 18a2 2 0 0 0 1.7 3h15a2 2 0 0 0 1.7-3L13.7 4.7a2 2 0 0 0-3.4 0Z"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <div>
        <p class="font-medium">yt-dlp {{ outdatedVersion }} is out of date</p>
        <p class="mt-1 text-[10px] text-amber-200/80">
          Update to reduce YouTube 403s:
          <code class="rounded bg-black/30 px-1">winget upgrade yt-dlp.yt-dlp</code>
        </p>
      </div>
    </div>
  </div>
</template>
