<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import { buildSelectionPayload } from "@/composables/selectionGeometry";
import {
  emitSelectionCancelled,
  emitSelectionResult,
  hideCurrentWindow
} from "@/composables/useWindowShell";

const isDragging = ref(false);
const startX = ref(0);
const startY = ref(0);
const currentX = ref(0);
const currentY = ref(0);
const selectionRoot = ref<HTMLElement | null>(null);
const monitorGeometry = ref({
  positionX: 0,
  positionY: 0,
  scaleFactor: 1
});

const selectionStyle = computed(() => {
  const x = Math.min(startX.value, currentX.value);
  const y = Math.min(startY.value, currentY.value);
  const width = Math.abs(currentX.value - startX.value);
  const height = Math.abs(currentY.value - startY.value);

  return {
    left: `${x}px`,
    top: `${y}px`,
    width: `${width}px`,
    height: `${height}px`
  };
});

const selectionDimensionsLabel = computed(() => {
  const width = Math.round(Math.abs(currentX.value - startX.value));
  const height = Math.round(Math.abs(currentY.value - startY.value));

  return `${width} × ${height}`;
});

function onMouseDown(event: MouseEvent) {
  if (event.button !== 0) return;
  void loadCursorMonitorGeometry();
  isDragging.value = true;
  startX.value = event.clientX;
  startY.value = event.clientY;
  currentX.value = event.clientX;
  currentY.value = event.clientY;
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging.value) return;
  currentX.value = event.clientX;
  currentY.value = event.clientY;
}

async function onMouseUp(event: MouseEvent) {
  if (!isDragging.value) return;
  isDragging.value = false;

  const width = Math.abs(event.clientX - startX.value);
  const height = Math.abs(event.clientY - startY.value);

  if (width < 4 || height < 4) {
    // Treat tiny rectangles as a click — cancel the selection.
    await cancel();
    return;
  }

  await loadCursorMonitorGeometry();

  const payload = buildSelectionPayload({
    startX: startX.value,
    startY: startY.value,
    endX: event.clientX,
    endY: event.clientY,
    monitor: monitorGeometry.value
  });

  // Hide before emitting so the overlay does not show up in the capture.
  await hideCurrentWindow();
  // Give the compositor a frame to redraw without the overlay.
  await new Promise((resolve) => setTimeout(resolve, 80));
  await emitSelectionResult(payload);
}

async function cancel() {
  isDragging.value = false;
  await hideCurrentWindow();
  await emitSelectionCancelled();
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    void cancel();
  }
}

async function loadCurrentMonitorGeometry() {
  try {
    const { currentMonitor } = await import("@tauri-apps/api/window");
    const monitor = await currentMonitor().catch(() => null);

    if (!monitor) {
      return;
    }

    monitorGeometry.value = {
      positionX: monitor.position.x,
      positionY: monitor.position.y,
      scaleFactor: monitor.scaleFactor || 1
    };
  } catch {
    // noop in web mode
  }
}

async function loadCursorMonitorGeometry() {
  try {
    const { cursorPosition, monitorFromPoint } = await import(
      "@tauri-apps/api/window"
    );
    const cursor = await cursorPosition().catch(() => null);

    if (!cursor) {
      return;
    }

    const monitor = await monitorFromPoint(cursor.x, cursor.y).catch(() => null);

    if (!monitor) {
      return;
    }

    monitorGeometry.value = {
      positionX: monitor.position.x,
      positionY: monitor.position.y,
      scaleFactor: monitor.scaleFactor || 1
    };
  } catch {
    // noop in web mode
  }
}

function focusSelectionRoot() {
  requestAnimationFrame(() => {
    selectionRoot.value?.focus();
  });
}

onMounted(async () => {
  await loadCurrentMonitorGeometry();
  await loadCursorMonitorGeometry();
  focusSelectionRoot();

  document.addEventListener("keydown", onKeyDown, true);
  window.addEventListener("focus", focusSelectionRoot);
});

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onKeyDown, true);
  window.removeEventListener("focus", focusSelectionRoot);
});
</script>

<template>
  <div
    ref="selectionRoot"
    class="apollo-selection-overlay"
    tabindex="0"
    @keydown.capture="onKeyDown"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
  >
    <div class="apollo-selection-backdrop" />

    <div
      v-if="isDragging"
      class="apollo-selection-rect"
      :style="selectionStyle"
    >
      <span class="apollo-selection-label">{{ selectionDimensionsLabel }}</span>
    </div>

    <div class="apollo-selection-hint">
      Arraste para selecionar uma área &nbsp;·&nbsp; Esc para cancelar
    </div>
  </div>
</template>

<style scoped>
.apollo-selection-overlay {
  position: fixed;
  inset: 0;
  cursor: crosshair;
  user-select: none;
  overflow: hidden;
  background: transparent !important;
}

.apollo-selection-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(15, 23, 42, 0.35);
  pointer-events: none;
}

.apollo-selection-rect {
  position: absolute;
  border: 2px solid rgb(56, 189, 248);
  background: rgba(56, 189, 248, 0.15);
  box-shadow: 0 0 0 9999px rgba(15, 23, 42, 0.45);
  pointer-events: none;
}

.apollo-selection-label {
  position: absolute;
  bottom: -1.75rem;
  right: 0;
  padding: 2px 8px;
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  color: #f8fafc;
  background: rgba(15, 23, 42, 0.85);
  border-radius: 4px;
  white-space: nowrap;
}

.apollo-selection-hint {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 16px;
  font-size: 13px;
  color: #f8fafc;
  background: rgba(15, 23, 42, 0.75);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: 9999px;
  pointer-events: none;
}
</style>
