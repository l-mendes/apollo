import { createApp } from "vue";

import App from "./App.vue";
import { apolloStore, apolloStoreKey } from "./store/apollo";
import TrayWindowApp from "./TrayWindowApp.vue";
import "./styles/main.css";

function resolveWindowHintFromLocation() {
  const windowHint = new URL(window.location.href).searchParams.get("window");

  if (
    windowHint === "tray" ||
    windowHint === "app" ||
    windowHint === "preview" ||
    windowHint === "response" ||
    windowHint === "selection"
  ) {
    return windowHint;
  }

  return null;
}

async function resolveRootComponent() {
  const windowHint = resolveWindowHintFromLocation();

  if (windowHint) {
    document.documentElement.dataset.window = windowHint;
    document.body.dataset.window = windowHint;

    if (windowHint === "tray") return TrayWindowApp;
    if (windowHint === "preview")
      return (await import("./PreviewWindow.vue")).default;
    if (windowHint === "response")
      return (await import("./ResponseWindow.vue")).default;
    if (windowHint === "selection")
      return (await import("./SelectionWindow.vue")).default;
    return App;
  }

  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const windowLabel = getCurrentWindow().label;

    document.documentElement.dataset.window = windowLabel;
    document.body.dataset.window = windowLabel;

    if (windowLabel === "tray") return TrayWindowApp;
    if (windowLabel === "preview")
      return (await import("./PreviewWindow.vue")).default;
    if (windowLabel === "response")
      return (await import("./ResponseWindow.vue")).default;
    if (windowLabel === "selection")
      return (await import("./SelectionWindow.vue")).default;
    return App;
  } catch {
    document.documentElement.dataset.window = "app";
    document.body.dataset.window = "app";

    return App;
  }
}

const RootComponent = await resolveRootComponent();

const app = createApp(RootComponent);

app.use(apolloStore, apolloStoreKey);
app.mount("#app");
