<script setup lang="ts">
import { computed } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();

const percentLabel = computed(() => `${Math.round(store.progress?.percent ?? 0)}%`);
</script>

<template>
  <div v-if="store.progress" class="space-y-1">
    <div class="flex items-center justify-between gap-2 text-xs">
      <span class="min-w-0 truncate text-slate-300">{{ store.progress.message }}</span>
      <span class="shrink-0 font-medium tabular-nums text-accent">{{ percentLabel }}</span>
    </div>
    <div class="h-2 overflow-hidden rounded-full bg-slate-700">
      <div
        class="h-full bg-accent transition-[width] duration-200 ease-out"
        :style="{ width: `${Math.min(100, Math.max(0, store.progress.percent))}%` }"
      />
    </div>
  </div>
</template>
