import { invoke } from "@tauri-apps/api/core";

export interface HealthStatus {
  appName: string;
  status: string;
  version: string;
}

export interface BootstrapMetadata {
  name: string;
  version: string;
  database_file_name: string;
}

export interface BootstrapSnapshot {
  metadata: BootstrapMetadata;
  database_path: string;
  applied_migrations: string[];
}

export async function fetchHealthStatus(): Promise<HealthStatus> {
  return invoke<HealthStatus>("health_check");
}

export async function fetchBootstrapSnapshot(): Promise<BootstrapSnapshot> {
  return invoke<BootstrapSnapshot>("bootstrap_summary");
}

export async function requestQuit(): Promise<void> {
  await invoke("quit_application");
}
