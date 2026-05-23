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

const mediaHeightClass = "max-h-[clamp(180px,36vh,360px)]";

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
    "Could not play in the built-in player. Check the saved path below.";
}
</script>

<template>
  <div class="min-w-0 overflow-hidden rounded-lg bg-black">
    <div
      class="flex w-full items-center justify-center bg-black"
      :class="mediaHeightClass"
    >
      <video
        v-if="store.canPlayInPlayer"
        ref="videoRef"
        :key="store.playbackKey"
        :src="store.playbackSrc!"
        :poster="posterUrl"
        :class="[mediaHeightClass, 'w-full object-contain']"
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
        :class="[mediaHeightClass, 'w-full object-contain']"
      />
      <div
        v-else
        class="flex h-[180px] w-full items-center justify-center text-xs text-slate-500"
      >
        Preview unavailable
      </div>
    </div>

    <p v-if="loadError" class="mt-1 line-clamp-2 text-xs text-red-400">
      {{ loadError }}
    </p>
    <p
      v-else-if="store.isYoutube && !store.canPlayInPlayer"
      class="mt-1 line-clamp-2 text-xs text-slate-500"
    >
      Thumbnail only. Download to preview the trimmed video here.
    </p>
  </div>
</template>
