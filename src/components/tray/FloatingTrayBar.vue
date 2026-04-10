<script setup lang="ts">
import { GripVertical, History, House, Power, Settings2 } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";

const props = defineProps<{
  versionText: string;
  activeSurface: "none" | "home" | "history" | "settings";
}>();

const emit = defineEmits<{
  home: [];
  history: [];
  settings: [];
  quit: [];
}>();

async function startDragging() {
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // noop in web mode
  }
}
</script>

<template>
  <div class="flex h-12 max-h-12 w-full items-center gap-2 rounded-3xl bg-apollo-tray-shell p-1 text-slate-50 backdrop-blur-xl">
    <button
      data-tauri-drag-region
      class="flex h-8 w-8 shrink-0 cursor-grab items-center justify-center rounded-full bg-apollo-tray-panel text-slate-400 transition hover:bg-apollo-tray-hover hover:text-slate-100 active:cursor-grabbing"
      aria-label="Drag tray"
      type="button"
      @click.prevent
      @mousedown.left.prevent="startDragging"
    >
      <GripVertical class="h-4 w-4" />
    </button>

    <div
      data-tauri-drag-region
      class="flex h-8 min-w-0 flex-1 cursor-grab items-center justify-between gap-2 rounded-2xl bg-apollo-tray-panel px-3 active:cursor-grabbing"
      @mousedown.left.prevent="startDragging"
    >
      <div class="flex min-w-0 items-center gap-2">
        <span
          class="h-2 w-2 shrink-0 rounded-full"
          :class="props.activeSurface === 'none' ? 'bg-apollo-tray-idle' : 'bg-apollo-tray-online shadow-trayStatus'"
        />
      </div>
      <p class="shrink-0 text-xs font-medium text-slate-200">
        {{ props.versionText }}
      </p>
    </div>

    <div class="h-6 w-px shrink-0 bg-apollo-tray-divider" />

    <div class="flex items-center gap-1.5">
      <button
        class="group relative flex h-8 w-8 items-center justify-center rounded-full transition hover:bg-apollo-tray-hover"
        :class="
          props.activeSurface === 'history'
            ? 'bg-apollo-tray-active text-ember-100'
            : 'bg-transparent text-slate-300'
        "
        aria-label="Open history"
        type="button"
        @mousedown.stop
        @click="emit('history')"
      >
        <History class="h-4 w-4" />
        <span class="pointer-events-none absolute -top-10 left-1/2 -translate-x-1/2 rounded-full bg-apollo-tray-tooltip px-3 py-1 text-xs text-slate-100 opacity-0 transition group-hover:opacity-100">
          Historico
        </span>
      </button>

      <button
        class="group relative flex h-8 w-8 items-center justify-center rounded-full transition hover:bg-apollo-tray-hover"
        :class="
          props.activeSurface === 'home'
            ? 'bg-apollo-tray-active text-ember-100'
            : 'bg-transparent text-slate-300'
        "
        aria-label="Open home"
        type="button"
        @mousedown.stop
        @click="emit('home')"
      >
        <House class="h-4 w-4" />
        <span class="pointer-events-none absolute -top-10 left-1/2 -translate-x-1/2 rounded-full bg-apollo-tray-tooltip px-3 py-1 text-xs text-slate-100 opacity-0 transition group-hover:opacity-100">
          Home
        </span>
      </button>

      <button
        class="group relative flex h-8 w-8 items-center justify-center rounded-full transition hover:bg-apollo-tray-hover"
        :class="
          props.activeSurface === 'settings'
            ? 'bg-apollo-tray-active text-ember-100'
            : 'bg-transparent text-slate-300'
        "
        aria-label="Open settings"
        type="button"
        @mousedown.stop
        @click="emit('settings')"
      >
        <Settings2 class="h-4 w-4" />
        <span class="pointer-events-none absolute -top-10 left-1/2 -translate-x-1/2 rounded-full bg-apollo-tray-tooltip px-3 py-1 text-xs text-slate-100 opacity-0 transition group-hover:opacity-100">
          Configuracoes
        </span>
      </button>

      <div class="h-6 w-px shrink-0 bg-apollo-tray-divider" />

      <button
        class="group relative flex h-8 w-8 items-center justify-center rounded-full bg-transparent text-slate-300 transition hover:bg-apollo-tray-danger hover:text-red-100"
        aria-label="Quit app"
        type="button"
        @mousedown.stop
        @click="emit('quit')"
      >
        <Power class="h-4 w-4" />
        <span class="pointer-events-none absolute -top-10 left-1/2 -translate-x-1/2 rounded-full bg-apollo-tray-tooltip px-3 py-1 text-xs text-slate-100 opacity-0 transition group-hover:opacity-100">
          Sair
        </span>
      </button>
    </div>
  </div>
</template>
