<script setup lang="ts">
import { useMediaStore } from "@/stores/mediaStore";
import ProgressBar from "@/components/ProgressBar.vue";

const store = useMediaStore();
</script>

<template>
  <div class="space-y-3">
    <h2 class="panel-title">
      <svg class="h-4 w-4 text-brand-400" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M12 4v10m0 0 4-4m-4 4-4-4M5 18h14"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      Export
    </h2>

    <ProgressBar v-if="store.exporting && store.progress" />

    <template v-if="store.isLocal">
      <button
        v-if="!store.isLocalAudioOnly"
        class="btn-primary w-full py-2 text-sm"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalTrimmed()"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M4 8.5A2.5 2.5 0 0 1 6.5 6h11A2.5 2.5 0 0 1 20 8.5v7A2.5 2.5 0 0 1 17.5 18h-11A2.5 2.5 0 0 1 4 15.5v-7Z"
            stroke="currentColor"
            stroke-width="1.6"
          />
          <path d="M10 9.8 15.5 12 10 14.2V9.8Z" fill="currentColor" />
        </svg>
        {{ store.exporting ? "Processing…" : "Trimmed video" }}
      </button>
      <button
        :class="store.isLocalAudioOnly ? 'btn-primary w-full py-2 text-sm' : 'btn-secondary w-full py-2 text-sm'"
        :disabled="!store.trimValidation.valid || store.exporting"
        @click="store.exportLocalAudio()"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M9 18V7.5l10-2V16"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <circle cx="7" cy="18" r="2.2" stroke="currentColor" stroke-width="1.7" />
          <circle cx="17" cy="16" r="2.2" stroke="currentColor" stroke-width="1.7" />
        </svg>
        {{ store.exporting ? "Processing…" : store.isLocalAudioOnly ? "Trimmed audio" : "Audio only" }}
      </button>
    </template>

    <template v-else-if="store.isYoutube">
      <div class="segmented">
        <button
          type="button"
          class="segmented-item"
          :class="store.formatFilter === 'video' ? 'segmented-item-active' : ''"
          :disabled="store.exporting"
          @click="store.setFormatFilter('video')"
        >
          Video
        </button>
        <button
          type="button"
          class="segmented-item"
          :class="store.formatFilter === 'audio' ? 'segmented-item-active' : ''"
          :disabled="store.exporting"
          @click="store.setFormatFilter('audio')"
        >
          Audio
        </button>
      </div>

      <div class="select-wrap">
        <select
          v-model="store.selectedFormatId"
          class="input-field py-2 text-xs"
          :disabled="store.exporting"
        >
          <option v-for="f in store.availableFormats" :key="f.format_id" :value="f.format_id">
            {{ f.label }}
          </option>
        </select>
        <svg class="select-chevron" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="m6 9 6 6 6-6"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </div>

      <button
        class="btn-primary w-full py-2 text-sm"
        :disabled="
          !store.trimValidation.valid || store.exporting || !store.selectedFormatId
        "
        @click="store.exportYoutube()"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 4v10m0 0 4-4m-4 4-4-4M5 18h14"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        {{ store.exporting ? "Processing…" : "Download (trimmed)" }}
      </button>
    </template>
  </div>
</template>
