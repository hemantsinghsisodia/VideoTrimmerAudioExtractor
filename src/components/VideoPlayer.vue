<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useMediaStore } from "@/stores/mediaStore";

const store = useMediaStore();
const videoRef = ref<HTMLVideoElement | null>(null);
const loadError = ref<string | null>(null);

const posterUrl = computed(() => store.thumbnailUrl ?? undefined);

const showYoutubeThumbnail = computed(
  () => store.isYoutube && !store.canPlayInPlayer && !!posterUrl.value,
);

watch(
  () => store.startSecs,
  (t) => {
    if (videoRef.value && Math.abs(videoRef.value.currentTime - t) > 0.5) {
      videoRef.value.currentTime = t;
    }
  },
);

watch(
  () => store.playbackKey,
  () => {
    loadError.value = null;
    if (videoRef.value) {
      videoRef.value.load();
    }
  },
);

function onTimeUpdate() {
  if (!videoRef.value) return;
  const t = videoRef.value.currentTime;
  if (t > store.endSecs) {
    videoRef.value.pause();
    videoRef.value.currentTime = store.startSecs;
  }
}

function onLoadedMetadata() {
  if (!videoRef.value) return;
  videoRef.value.currentTime = store.startSecs;
}

function onVideoError() {
  loadError.value =
    "Could not play this file in the built-in player. It may still be saved on disk — check the path below.";
}
</script>

<template>
  <div class="min-w-0 overflow-hidden rounded-lg bg-black">
    <div class="flex max-h-[min(60vh,480px)] w-full items-center justify-center bg-black">
      <video
        v-if="store.canPlayInPlayer"
        ref="videoRef"
        :key="store.playbackKey"
        :src="store.playbackSrc!"
        :poster="posterUrl"
        class="max-h-[min(60vh,480px)] w-full object-contain"
        controls
        preload="auto"
        @timeupdate="onTimeUpdate"
        @loadedmetadata="onLoadedMetadata"
        @error="onVideoError"
      />
      <img
        v-else-if="showYoutubeThumbnail || posterUrl"
        :src="posterUrl"
        alt="Video thumbnail"
        class="max-h-[min(60vh,480px)] w-full object-contain"
      />
      <div
        v-else
        class="flex aspect-video w-full max-w-full items-center justify-center text-sm text-slate-500"
      >
        Preview unavailable
      </div>
    </div>

    <p v-if="loadError" class="mt-2 break-words text-xs text-red-400">
      {{ loadError }}
    </p>
    <p
      v-else-if="store.isYoutube && !store.canPlayInPlayer"
      class="mt-2 break-words text-xs text-slate-500"
    >
      Thumbnail preview only. Trim on the timeline, then download to play the exported video here.
    </p>
  </div>
</template>
