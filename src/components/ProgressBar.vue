<script setup lang="ts">
import { computed } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();

const percentLabel = computed(() => `${Math.round(store.progress?.percent ?? 0)}%`);
</script>

<template>
  <div v-if="store.progress" class="space-y-1.5">
    <div class="flex items-center justify-between gap-2 text-xs">
      <span class="min-w-0 truncate text-slate-300">{{ store.progress.message }}</span>
      <span class="shrink-0 font-medium tabular-nums text-brand-400">{{ percentLabel }}</span>
    </div>
    <div class="relative h-2 overflow-hidden rounded-full bg-white/10">
      <div
        class="h-full rounded-full bg-gradient-to-r from-brand-600 to-brand-400 transition-[width] duration-200 ease-out"
        :style="{ width: `${Math.min(100, Math.max(0, store.progress.percent))}%` }"
      />
      <div
        class="pointer-events-none absolute inset-y-0 w-1/3 bg-gradient-to-r from-transparent via-white/30 to-transparent animate-shimmer"
      />
    </div>
  </div>
</template>
