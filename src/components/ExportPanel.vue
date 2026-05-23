<script setup lang="ts">
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();
</script>

<template>
  <div class="space-y-2">
    <h2 class="text-sm font-semibold">Export</h2>

    <div v-if="store.progress" class="space-y-1">
      <div class="h-1.5 overflow-hidden rounded-full bg-slate-700">
        <div
          class="h-full bg-accent transition-all"
          :style="{ width: `${store.progress.percent}%` }"
        />
      </div>
      <p class="line-clamp-2 text-[10px] text-slate-400">{{ store.progress.message }}</p>
    </div>

    <template v-if="store.isLocal">
      <button
        class="btn-primary w-full py-1.5 text-sm"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalTrimmed()"
      >
        Trimmed video
      </button>
      <button
        class="btn-secondary w-full py-1.5 text-sm"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalAudio()"
      >
        Audio only
      </button>
    </template>

    <template v-else-if="store.isYoutube">
      <div class="flex flex-wrap gap-1 text-[10px]">
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'all' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'all'"
        >
          Recommended
        </button>
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'video' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'video'"
        >
          Video
        </button>
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'audio' ? 'bg-accent text-white' : 'bg-slate-700'"
          @click="store.formatFilter = 'audio'"
        >
          Audio
        </button>
      </div>

      <select v-model="store.selectedFormatId" class="input-field py-1 text-xs">
        <option v-for="f in store.availableFormats" :key="f.format_id" :value="f.format_id">
          {{ f.label }}
        </option>
      </select>

      <button
        class="btn-primary w-full py-1.5 text-sm"
        :disabled="
          !store.trimValidation.valid || store.exporting || !store.selectedFormatId
        "
        @click="store.exportYoutube()"
      >
        {{ store.exporting ? "Downloading…" : "Download (trimmed)" }}
      </button>
    </template>
  </div>
</template>
