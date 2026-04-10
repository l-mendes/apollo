declare module "vuex" {
  import type { App, InjectionKey } from "vue";

  export class Store<S> {
    readonly state: S;
    readonly getters: Record<string, unknown>;

    install(app: App, injectKey?: InjectionKey<Store<any>> | string): void;
    commit(type: string, payload?: any): void;
  }

  export interface StoreOptions<S> {
    state?: S | (() => S);
    getters?: Record<
      string,
      (
        state: S,
        getters: Record<string, unknown>,
        rootState: S,
        rootGetters: Record<string, unknown>
      ) => unknown
    >;
    mutations?: Record<string, (state: S, payload?: any) => void>;
  }

  export function createStore<S>(options: StoreOptions<S>): Store<S>;
  export function useStore<S = any>(
    injectKey?: InjectionKey<Store<S>> | string
  ): Store<S>;
}
