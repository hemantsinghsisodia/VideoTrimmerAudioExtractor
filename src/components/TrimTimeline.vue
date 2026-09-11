<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useMediaStore } from "@/stores/mediaStore";
import { clamp, formatTime } from "@/utils/time";

const store = useMediaStore();
const trackRef = ref<HTMLDivElement | null>(null);
const dragging = ref<"start" | "end" | "range" | null>(null);
const dragOffset = ref(0);

const duration = computed(() => Math.max(store.duration, 0.001));

const startPercent = computed(() => (store.startSecs / duration.value) * 100);
const endPercent = computed(() => (store.endSecs / duration.value) * 100);
const playheadPercent = computed(() => startPercent.value);

function secsFromClientX(clientX: number): number {
  const el = trackRef.value;
  if (!el) return 0;
  const rect = el.getBoundingClientRect();
  const ratio = clamp((clientX - rect.left) / rect.width, 0, 1);
  return ratio * duration.value;
}

function onPointerDown(handle: "start" | "end", e: PointerEvent) {
  dragging.value = handle;
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
}

function onRangePointerDown(e: PointerEvent) {
  const startX = secsFromClientX(e.clientX);
  dragOffset.value = startX - store.startSecs;
  dragging.value = "range";
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  const t = secsFromClientX(e.clientX);

  if (dragging.value === "start") {
    const newStart = clamp(t, 0, store.endSecs - 0.1);
    store.setTrimFromTimeline(newStart, store.endSecs);
  } else if (dragging.value === "end") {
    const newEnd = clamp(t, store.startSecs + 0.1, duration.value);
    store.setTrimFromTimeline(store.startSecs, newEnd);
  } else if (dragging.value === "range") {
    const len = store.endSecs - store.startSecs;
    let newStart = t - dragOffset.value;
    newStart = clamp(newStart, 0, duration.value - len);
    store.setTrimFromTimeline(newStart, newStart + len);
  }
}

function onPointerUp() {
  dragging.value = null;
}

function onTrackClick(e: MouseEvent) {
  if (dragging.value) return;
  const t = secsFromClientX(e.clientX);
  const distStart = Math.abs(t - store.startSecs);
  const distEnd = Math.abs(t - store.endSecs);
  if (distStart < distEnd) {
    store.setTrimFromTimeline(clamp(t, 0, store.endSecs - 0.1), store.endSecs);
  } else {
    store.setTrimFromTimeline(store.startSecs, clamp(t, store.startSecs + 0.1, duration.value));
  }
}

onMounted(() => {
  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
});

onUnmounted(() => {
  window.removeEventListener("pointermove", onPointerMove);
  window.removeEventListener("pointerup", onPointerUp);
});
</script>

<template>
  <div class="space-y-1.5">
    <div class="flex justify-between text-[10px]">
      <span class="rounded-full bg-white/5 px-2 py-0.5 tabular-nums text-slate-400">0:00</span>
      <span class="rounded-full bg-white/5 px-2 py-0.5 tabular-nums text-slate-400">
        {{ formatTime(duration) }}
      </span>
    </div>
    <div
      ref="trackRef"
      class="relative h-10 cursor-pointer rounded-xl bg-black/30 ring-1 ring-white/10"
      @click="onTrackClick"
    >
      <div class="absolute inset-y-2 rounded-full bg-white/10" style="left: 0; right: 0" />
      <div
        class="absolute inset-y-2 rounded-full bg-gradient-to-r from-brand-600/80 to-brand-400/80"
        :style="{ left: `${startPercent}%`, width: `${endPercent - startPercent}%` }"
        @pointerdown="onRangePointerDown"
      />
      <div
        class="absolute top-1 z-10 h-8 w-3 -translate-x-1/2 cursor-grab rounded-full bg-emerald-400 shadow-glow active:cursor-grabbing"
        :style="{ left: `${startPercent}%` }"
        @pointerdown="onPointerDown('start', $event)"
      >
        <span class="absolute inset-y-2 left-1/2 w-px -translate-x-1/2 bg-emerald-950/40" />
      </div>
      <div
        class="absolute top-1 z-10 h-8 w-3 -translate-x-1/2 cursor-grab rounded-full bg-rose-400 shadow-glow active:cursor-grabbing"
        :style="{ left: `${endPercent}%` }"
        @pointerdown="onPointerDown('end', $event)"
      >
        <span class="absolute inset-y-2 left-1/2 w-px -translate-x-1/2 bg-rose-950/40" />
      </div>
      <div
        class="absolute top-1/2 z-0 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white/80"
        :style="{ left: `${playheadPercent}%` }"
      />
    </div>
    <p class="text-center text-[10px] text-slate-500">
      Drag handles or click track
    </p>
  </div>
</template>
