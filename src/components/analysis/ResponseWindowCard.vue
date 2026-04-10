<script setup lang="ts">
import { computed } from "vue";

import { providerLabel, type NormalizedResponse } from "@/composables/useApolloDesktop";

const props = defineProps<{
  response: NormalizedResponse | null;
  requestPrompt: string;
  loading: boolean;
  errorText: string | null;
}>();

const providerText = computed(() => {
  if (!props.response) {
    return "Sem resposta";
  }

  return `${providerLabel(props.response.provider_kind)} · ${props.response.model_key}`;
});
</script>

<template>
  <section
    class="overflow-hidden rounded-xl border border-apollo-app-border bg-apollo-app-card"
    data-testid="response-card"
  >
    <div class="flex items-center justify-between border-b border-apollo-app-border px-5 py-4">
      <div>
        <p class="text-sm font-semibold text-white">Resposta do Apollo</p>
        <p class="mt-0.5 text-xs text-apollo-app-muted">Saida normalizada</p>
      </div>
      <span class="rounded-lg border border-apollo-app-border bg-apollo-app-shell px-2.5 py-1 text-xs text-slate-200">
        {{ providerText }}
      </span>
    </div>

    <div class="space-y-4 p-5">
      <div
        v-if="props.loading"
        class="rounded-lg border border-apollo-app-selectedBorder bg-apollo-app-selected p-4 text-sm text-slate-100"
      >
        O provider esta processando o contexto enviado.
      </div>

      <div
        v-else-if="props.errorText"
        class="rounded-lg border border-red-400/30 bg-red-500/10 p-4 text-sm text-red-100"
      >
        {{ props.errorText }}
      </div>

      <template v-else-if="props.response">
        <div class="rounded-lg border border-apollo-app-border bg-apollo-app-shell p-4">
          <p class="text-xs font-medium text-apollo-app-muted uppercase">Resposta</p>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-50">
            {{ props.response.answer }}
          </p>
        </div>

        <div
          v-if="props.requestPrompt"
          class="rounded-lg border border-apollo-app-border bg-apollo-app-shell p-4"
        >
          <p class="text-xs font-medium text-apollo-app-muted uppercase">Prompt composto</p>
          <p class="mt-2 max-h-28 overflow-y-auto whitespace-pre-wrap text-sm leading-6 text-slate-200">
            {{ props.requestPrompt }}
          </p>
        </div>
      </template>

      <div
        v-else
        class="rounded-lg border border-dashed border-apollo-app-border bg-apollo-app-shell p-4 text-sm text-slate-300"
      >
        A resposta mais recente aparece aqui com provider, modelo e prompt efetivo.
      </div>
    </div>
  </section>
</template>