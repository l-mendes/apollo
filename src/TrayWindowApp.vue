<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import FloatingTrayBar from "@/components/tray/FloatingTrayBar.vue";
import type { SurfaceId } from "@/composables/useApolloDesktop";
import { type AppSurface, listenForSurfaceChanged, openAppWindow, syncTrayWindowAppearance } from "@/composables/useWindowShell";
import {
  fetchBootstrapSnapshot,
  fetchHealthStatus,
  requestQuit,
  type BootstrapSnapshot,
  type HealthStatus
} from "@/composables/useDesktopCapabilities";

const activeSurface = ref<SurfaceId>("none");
const health = ref<HealthStatus | null>(null);
const snapshot = ref<BootstrapSnapshot | null>(null);

let unlistenSurfaceChanged: (() => void) | null = null;

const versionText = computed(() => health.value?.version ?? snapshot.value?.metadata.version ?? "v0.1.0");

onMounted(async () => {
  await syncTrayWindowAppearance();
  unlistenSurfaceChanged = await listenForSurfaceChanged((surface) => {
    activeSurface.value = surface;
  });

  try {
    const [healthStatus, bootstrap] = await Promise.all([
      fetchHealthStatus(),
      fetchBootstrapSnapshot()
    ]);

    health.value = healthStatus;
    snapshot.value = bootstrap;
  } catch {
    health.value = {
      appName: "Apollo",
      status: "Modo web para desenvolvimento",
      version: "v0.1.0"
    };
  }
});

onBeforeUnmount(() => {
  unlistenSurfaceChanged?.();
});

async function handleOpen(surface: AppSurface) {
  activeSurface.value = surface;
  await openAppWindow(surface);
}

async function handleQuit() {
  try {
    await requestQuit();
  } catch {
    activeSurface.value = "none";
  }
}
</script>

<template>
  <div class="apollo-tray-shell">
    <FloatingTrayBar
      :active-surface="activeSurface"
      :version-text="versionText"
      @home="handleOpen('home')"
      @history="handleOpen('history')"
      @settings="handleOpen('settings')"
      @quit="handleQuit"
    />
  </div>
</template>