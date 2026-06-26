<script setup lang="ts">
import { useMediaStore } from "@/stores/mediaStore";
import ProgressBar from "@/components/ProgressBar.vue";

const store = useMediaStore();
</script>

<template>
  <div class="space-y-2">
    <h2 class="text-sm font-semibold">Export</h2>

    <ProgressBar v-if="store.exporting && store.progress" />

    <template v-if="store.isLocal">
      <button
        v-if="!store.isLocalAudioOnly"
        class="btn-primary w-full py-1.5 text-sm"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalTrimmed()"
      >
        {{ store.exporting ? "Processing…" : "Trimmed video" }}
      </button>
      <button
        :class="store.isLocalAudioOnly ? 'btn-primary w-full py-1.5 text-sm' : 'btn-secondary w-full py-1.5 text-sm'"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalAudio()"
      >
        {{ store.exporting ? "Processing…" : store.isLocalAudioOnly ? "Trimmed audio" : "Audio only" }}
      </button>
    </template>

    <template v-else-if="store.isYoutube">
      <div class="flex flex-wrap gap-1 text-[10px]">
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'all' ? 'bg-accent text-white' : 'bg-slate-700'"
          :disabled="store.exporting"
          @click="store.formatFilter = 'all'"
        >
          Recommended
        </button>
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'video' ? 'bg-accent text-white' : 'bg-slate-700'"
          :disabled="store.exporting"
          @click="store.formatFilter = 'video'"
        >
          Video
        </button>
        <button
          class="rounded px-2 py-0.5"
          :class="store.formatFilter === 'audio' ? 'bg-accent text-white' : 'bg-slate-700'"
          :disabled="store.exporting"
          @click="store.formatFilter = 'audio'"
        >
          Audio
        </button>
      </div>

      <select
        v-model="store.selectedFormatId"
        class="input-field py-1 text-xs"
        :disabled="store.exporting"
      >
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
        {{ store.exporting ? "Processing…" : "Download (trimmed)" }}
      </button>
    </template>
  </div>
</template>
