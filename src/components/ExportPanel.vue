<script setup lang="ts">
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();
</script>

<template>
  <div class="space-y-4">
    <h2 class="text-lg font-semibold">Export</h2>

    <div v-if="store.progress" class="space-y-1">
      <div class="h-2 overflow-hidden rounded-full bg-slate-700">
        <div
          class="h-full bg-accent transition-all"
          :style="{ width: `${store.progress.percent}%` }"
        />
      </div>
      <p class="text-xs text-slate-400">{{ store.progress.message }}</p>
    </div>

    <template v-if="store.isLocal">
      <p class="text-sm text-slate-400">Local file exports</p>
      <button
        class="btn-primary w-full"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalTrimmed()"
      >
        Download trimmed video
      </button>
      <button
        class="btn-secondary w-full"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalAudio()"
      >
        Download audio only (best quality)
      </button>
    </template>

    <template v-else-if="store.isYoutube">
      <p class="text-sm text-slate-400">YouTube format &amp; quality</p>

      <div class="flex gap-2 text-xs">
        <button
          class="rounded px-2 py-1"
          :class="store.formatFilter === 'all' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'all'"
        >
          Recommended
        </button>
        <button
          class="rounded px-2 py-1"
          :class="store.formatFilter === 'video' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'video'"
        >
          Video
        </button>
        <button
          class="rounded px-2 py-1"
          :class="store.formatFilter === 'audio' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'audio'"
        >
          Audio
        </button>
      </div>

      <p class="text-xs text-slate-500">
        Pick a quality below. Combined MP4 options download fastest; video-only may take longer.
      </p>
      <select v-model="store.selectedFormatId" class="input-field max-h-48 text-xs">
        <option v-for="f in store.availableFormats" :key="f.format_id" :value="f.format_id">
          {{ f.label }}
        </option>
      </select>

      <button
        class="btn-primary w-full"
        :disabled="
          !store.trimValidation.valid || store.exporting || !store.selectedFormatId
        "
        @click="store.exportYoutube()"
      >
        {{ store.exporting ? "Downloading…" : "Download selected format (trimmed)" }}
      </button>
    </template>
  </div>
</template>
