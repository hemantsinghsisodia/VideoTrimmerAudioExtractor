<script setup lang="ts">
import { useMediaStore } from "@/stores/mediaStore";
import { formatTime } from "@/utils/time";

const store = useMediaStore();

function apply() {
  store.applyTrimFromInputs();
}
</script>

<template>
  <div class="grid gap-4 sm:grid-cols-2">
    <label class="block space-y-1 text-sm">
      <span class="text-slate-400">Start (MM:SS or HH:MM:SS)</span>
      <input
        v-model="store.startInput"
        class="input-field"
        placeholder="00:00"
        @change="apply"
        @keydown.enter="apply"
      />
    </label>
    <label class="block space-y-1 text-sm">
      <span class="text-slate-400">End (max {{ formatTime(store.duration) }})</span>
      <input
        v-model="store.endInput"
        class="input-field"
        placeholder="00:00"
        @change="apply"
        @keydown.enter="apply"
      />
    </label>
  </div>
  <p
    class="text-xs"
    :class="store.trimValidation.valid ? 'text-slate-500' : 'text-red-400'"
  >
    {{
      store.trimValidation.valid
        ? `Selection: ${formatTime(store.startSecs)} – ${formatTime(store.endSecs)} (${(store.endSecs - store.startSecs).toFixed(1)}s)`
        : store.trimValidation.error
    }}
  </p>
</template>
