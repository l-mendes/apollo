import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn()
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

import {
  fetchBootstrapSnapshot,
  fetchHealthStatus,
  requestQuit
} from "@/composables/useDesktopCapabilities";

describe("useDesktopCapabilities", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("delegates health checks to the Tauri backend", async () => {
    invokeMock.mockResolvedValue({
      appName: "Apollo",
      status: "ready",
      version: "v0.1.0"
    });

    const result = await fetchHealthStatus();

    expect(result.status).toBe("ready");
    expect(invokeMock).toHaveBeenCalledWith("health_check");
  });

  it("requests the bootstrap summary from Tauri", async () => {
    invokeMock.mockResolvedValue({
      metadata: {
        name: "Apollo",
        version: "v0.1.0",
        database_file_name: "apollo.db"
      },
      database_path: "/tmp/apollo.db",
      applied_migrations: ["0001_initial_schema"]
    });

    const result = await fetchBootstrapSnapshot();

    expect(result.applied_migrations).toContain("0001_initial_schema");
    expect(invokeMock).toHaveBeenCalledWith("bootstrap_summary");
  });

  it("sends the quit command when requested", async () => {
    invokeMock.mockResolvedValue(undefined);

    await requestQuit();

    expect(invokeMock).toHaveBeenCalledWith("quit_application");
  });
});
