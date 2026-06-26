import { defineStore } from "pinia";
import { computed, nextTick, ref } from "vue";
import type {
  JobProgress,
  MediaProbe,
  SourceType,
  YoutubeFormat,
  YoutubeInfo,
} from "@/types/media";
import {
  cancelJob,
  checkDependencies,
  downloadYoutube,
  extractAudio,
  getYoutubeFormats,
  onJobProgress,
  pickSavePath,
  pickVideoFile,
  probeLocalFile,
  stageForPlayback,
  trimVideo,
  videoSrcFromPath,
} from "@/services/tauri";
import { validateTrimRange, formatTime, parseTimeInput } from "@/utils/time";
import { validateYoutubeUrl } from "@/utils/youtube";
import { getUserFacingFormats, pickDefaultFormatId, resolveYoutubeDownloadFormat } from "@/utils/formats";
import { isCancelledError } from "@/utils/progress";

export const useMediaStore = defineStore("media", () => {
  const sourceType = ref<SourceType | null>(null);
  const localPath = ref<string | null>(null);
  const youtubeUrl = ref("");
  const probe = ref<MediaProbe | null>(null);
  const youtubeInfo = ref<YoutubeInfo | null>(null);
  const previewPath = ref<string | null>(null);
  const downloadedPath = ref<string | null>(null);
  const playbackKey = ref(0);

  const startSecs = ref(0);
  const endSecs = ref(0);
  const startInput = ref("00:00");
  const endInput = ref("00:00");

  const selectedFormatId = ref<string | null>(null);
  const formatFilter = ref<"all" | "video" | "audio">("all");

  const loading = ref(false);
  const exporting = ref(false);
  const cancelling = ref(false);
  const error = ref<string | null>(null);
  const progress = ref<JobProgress | null>(null);
  const lastOutputPath = ref<string | null>(null);

  const deps = ref<{ ffmpeg: boolean; ffprobe: boolean; ytdlp: boolean; messages: string[] } | null>(
    null,
  );

  const duration = computed(() => probe.value?.duration_secs ?? youtubeInfo.value?.duration_secs ?? 0);

  const trimValidation = computed(() =>
    validateTrimRange(startSecs.value, endSecs.value, duration.value),
  );

  const availableFormats = computed(() => {
    if (!youtubeInfo.value) return [] as YoutubeFormat[];
    return getUserFacingFormats(youtubeInfo.value.formats, formatFilter.value);
  });

  const isYoutube = computed(() => sourceType.value === "youtube");
  const isLocal = computed(() => sourceType.value === "local");
  const hasSource = computed(() => !!probe.value || !!youtubeInfo.value);

  const thumbnailUrl = computed(() => youtubeInfo.value?.thumbnail ?? null);

  const playbackSrc = computed(() => {
    if (downloadedPath.value) return videoSrcFromPath(downloadedPath.value);
    if (isLocal.value && localPath.value) return videoSrcFromPath(localPath.value);
    return null;
  });

  const canPlayInPlayer = computed(() => !!playbackSrc.value);

  async function setPlaybackFromFile(path: string) {
    const staged = await stageForPlayback(path);
    downloadedPath.value = null;
    await nextTick();
    downloadedPath.value = staged;
    playbackKey.value += 1;
  }

  function scheduleClearProgress() {
    setTimeout(() => {
      if (!exporting.value && !loading.value) {
        progress.value = null;
      }
    }, 1500);
  }

  async function init() {
    deps.value = await checkDependencies();
    await onJobProgress((p) => {
      progress.value = p;
    });
  }

  async function cancelCurrentJob() {
    if (!loading.value && !exporting.value) return;
    cancelling.value = true;
    error.value = null;
    try {
      await cancelJob();
    } finally {
      cancelling.value = false;
    }
  }

  function handleJobError(e: unknown) {
    if (isCancelledError(e)) {
      error.value = null;
      progress.value = {
        job_id: "default",
        percent: 0,
        message: "Cancelled",
      };
      return;
    }
    error.value = e instanceof Error ? e.message : String(e);
  }

  function reset() {
    sourceType.value = null;
    localPath.value = null;
    youtubeUrl.value = "";
    probe.value = null;
    youtubeInfo.value = null;
    previewPath.value = null;
    downloadedPath.value = null;
    startSecs.value = 0;
    endSecs.value = 0;
    startInput.value = "00:00";
    endInput.value = "00:00";
    selectedFormatId.value = null;
    error.value = null;
    progress.value = null;
    lastOutputPath.value = null;
  }

  function syncInputsFromSecs() {
    startInput.value = formatTime(startSecs.value);
    endInput.value = formatTime(endSecs.value);
  }

  function applyTrimFromInputs() {
    const s = parseTimeInput(startInput.value);
    const e = parseTimeInput(endInput.value);
    if (s === null || e === null) {
      error.value = "Invalid time format. Use MM:SS or HH:MM:SS";
      return false;
    }
    const v = validateTrimRange(s, e, duration.value);
    if (!v.valid) {
      error.value = v.error ?? "Invalid trim range";
      return false;
    }
    startSecs.value = v.start;
    endSecs.value = v.end;
    error.value = null;
    return true;
  }

  function setTrimFromTimeline(start: number, end: number) {
    startSecs.value = start;
    endSecs.value = end;
    syncInputsFromSecs();
  }

  async function loadLocalFile(path?: string) {
    loading.value = true;
    error.value = null;
    try {
      const filePath = path ?? (await pickVideoFile());
      if (!filePath) return;

      const result = await probeLocalFile(filePath);
      sourceType.value = "local";
      localPath.value = filePath;
      probe.value = result;
      youtubeInfo.value = null;
      previewPath.value = filePath;
      await setPlaybackFromFile(filePath);
      endSecs.value = result.duration_secs;
      endInput.value = formatTime(result.duration_secs);
      startSecs.value = 0;
      startInput.value = "00:00";
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadYoutube(url: string) {
    const validation = validateYoutubeUrl(url);
    if (!validation.valid) {
      error.value = validation.error ?? "Invalid YouTube URL";
      return;
    }

    loading.value = true;
    error.value = null;
    progress.value = { job_id: "default", percent: 0, message: "Fetching video info…" };
    try {
      const info = await getYoutubeFormats(url);
      sourceType.value = "youtube";
      youtubeUrl.value = url;
      youtubeInfo.value = info;
      localPath.value = null;
      probe.value = {
        path: "",
        duration_secs: info.duration_secs,
        title: info.title,
      };
      previewPath.value = info.thumbnail ?? null;
      endSecs.value = info.duration_secs;
      endInput.value = formatTime(info.duration_secs);
      startSecs.value = 0;
      startInput.value = "00:00";
      selectedFormatId.value = pickDefaultFormatId(info.formats);
      downloadedPath.value = null;
    } catch (e) {
      handleJobError(e);
    } finally {
      loading.value = false;
      if (!exporting.value) scheduleClearProgress();
    }
  }

  async function exportLocalTrimmed() {
    if (!localPath.value || !applyTrimFromInputs()) return;
    const defaultName = `trimmed_${Date.now()}.mp4`;
    const outputPath = await pickSavePath(defaultName);
    if (!outputPath) return;

    exporting.value = true;
    error.value = null;
    progress.value = { job_id: "default", percent: 0, message: "Preparing trim…" };
    try {
      const result = await trimVideo(
        localPath.value,
        outputPath,
        startSecs.value,
        endSecs.value,
        false,
      );
      lastOutputPath.value = result.output_path;
      await setPlaybackFromFile(result.output_path);
    } catch (e) {
      handleJobError(e);
    } finally {
      exporting.value = false;
      scheduleClearProgress();
    }
  }

  async function exportLocalAudio() {
    if (!localPath.value || !applyTrimFromInputs()) return;
    const defaultName = `audio_${Date.now()}.m4a`;
    const outputPath = await pickSavePath(defaultName);
    if (!outputPath) return;

    exporting.value = true;
    error.value = null;
    progress.value = { job_id: "default", percent: 0, message: "Preparing audio export…" };
    try {
      const result = await extractAudio(
        localPath.value,
        outputPath,
        startSecs.value,
        endSecs.value,
      );
      lastOutputPath.value = result.output_path;
    } catch (e) {
      handleJobError(e);
    } finally {
      exporting.value = false;
      scheduleClearProgress();
    }
  }

  async function exportYoutube() {
    if (!youtubeUrl.value || !selectedFormatId.value || !applyTrimFromInputs()) return;
    const selected = availableFormats.value.find(
      (f) => f.format_id === selectedFormatId.value,
    );
    if (!selected) return;

    const download = resolveYoutubeDownloadFormat(selected);
    const defaultName = `youtube_${Date.now()}.${download.defaultExtension}`;
    const outputPath = await pickSavePath(defaultName);
    if (!outputPath) return;

    exporting.value = true;
    error.value = null;
    progress.value = { job_id: "default", percent: 0, message: "Preparing download…" };
    try {
      const result = await downloadYoutube({
        url: youtubeUrl.value,
        formatId: download.formatId,
        outputPath,
        startSecs: startSecs.value,
        endSecs: endSecs.value,
        videoOnly: download.videoOnly,
        audioOnly: download.audioOnly,
        convertTo: download.convertTo,
        audioQuality: download.audioQuality,
      });
      lastOutputPath.value = result.output_path;
      if (!selected.audio_only) {
        await setPlaybackFromFile(result.output_path);
      }
    } catch (e) {
      handleJobError(e);
    } finally {
      exporting.value = false;
      scheduleClearProgress();
    }
  }

  return {
    sourceType,
    localPath,
    youtubeUrl,
    probe,
    youtubeInfo,
    previewPath,
    downloadedPath,
    thumbnailUrl,
    playbackSrc,
    canPlayInPlayer,
    playbackKey,
    startSecs,
    endSecs,
    startInput,
    endInput,
    selectedFormatId,
    formatFilter,
    loading,
    exporting,
    cancelling,
    error,
    progress,
    lastOutputPath,
    deps,
    duration,
    trimValidation,
    availableFormats,
    isYoutube,
    isLocal,
    hasSource,
    init,
    reset,
    syncInputsFromSecs,
    applyTrimFromInputs,
    setTrimFromTimeline,
    loadLocalFile,
    loadYoutube,
    exportLocalTrimmed,
    exportLocalAudio,
    exportYoutube,
    cancelCurrentJob,
  };
});
