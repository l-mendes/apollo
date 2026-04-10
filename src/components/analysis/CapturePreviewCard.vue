<script setup lang="ts">
const props = defineProps<{
  ocrText: string;
  userNotes: string;
}>();
</script>

<template>
  <section
    class="overflow-hidden rounded-xl border border-apollo-app-border bg-apollo-app-card"
    data-testid="preview-card"
  >
    <div class="flex items-center justify-between border-b border-apollo-app-border px-5 py-4">
      <div>
        <p class="text-sm font-semibold text-white">Preview do contexto</p>
        <p class="mt-0.5 text-xs text-apollo-app-muted">Dados que seguem para a analise</p>
      </div>
      <span class="rounded-lg border border-emerald-400/20 bg-emerald-400/10 px-2 py-1 text-xs text-emerald-100">
        Local
      </span>
    </div>

    <div class="space-y-4 p-5">
      <div class="rounded-lg border border-apollo-app-border bg-apollo-app-shell p-4">
        <p class="text-xs font-medium text-apollo-app-muted uppercase">OCR</p>
        <p class="mt-2 max-h-32 overflow-y-auto whitespace-pre-wrap text-sm leading-6 text-slate-100">
          {{ props.ocrText.trim() || "O texto capturado aparece aqui assim que voce colar ou descrever o trecho da tela." }}
        </p>
      </div>

      <div class="rounded-lg border border-apollo-app-border bg-apollo-app-shell p-4">
        <p class="text-xs font-medium text-apollo-app-muted uppercase">Notas do usuario</p>
        <p class="mt-2 max-h-24 overflow-y-auto whitespace-pre-wrap text-sm leading-6 text-slate-200">
          {{ props.userNotes.trim() || "Use notas para direcionar nuances, duvidas de gramatica ou o contexto pragmatico." }}
        </p>
      </div>
    </div>
  </section>
</template>