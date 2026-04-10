use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use tauri_plugin_global_shortcut::Shortcut;

use crate::{
    application::{
        bootstrap_snapshot::BootstrapSnapshot,
        use_cases::{
            analyze_capture_service::AnalyzeCaptureService,
            continue_conversation_service::ContinueConversationService,
            list_provider_models_service::ListProviderModelsService,
        },
    },
    domain::entities::shortcut_binding::ShortcutBinding,
    infrastructure::{
        ocr::TesseractOcrEngine,
        persistence::SqliteAppRepository,
        provider_catalog::ManualProviderModelCatalog,
        provider_registry::InMemoryProviderRegistry,
        providers::{
            cli::{command_runner::ProcessCommandRunner, executor::GenericCliProviderExecutor},
            http::{
                adapters::{AnthropicProvider, OllamaProvider, OpenAiProvider},
                transport::ReqwestHttpTransport,
            },
            runtime_profiles::{
                default_cli_profiles, default_ollama_cloud_endpoint, default_ollama_local_endpoint,
            },
        },
    },
};

#[derive(Clone)]
pub struct AppState {
    snapshot: BootstrapSnapshot,
    repository: Arc<SqliteAppRepository>,
    model_catalog: Arc<ManualProviderModelCatalog>,
    provider_registry: Arc<InMemoryProviderRegistry>,
    list_provider_models: Arc<ListProviderModelsService>,
    analyze_capture: Arc<AnalyzeCaptureService>,
    continue_conversation: Arc<ContinueConversationService>,
    ocr_engine: Arc<TesseractOcrEngine>,
    shortcut_bindings: Arc<RwLock<Vec<ShortcutBinding>>>,
    ocr_language: Arc<RwLock<String>>,
}

impl AppState {
    pub fn new(snapshot: BootstrapSnapshot) -> Self {
        let repository = Arc::new(SqliteAppRepository::new(PathBuf::from(
            snapshot.database_path.clone(),
        )));
        let model_catalog = Arc::new(ManualProviderModelCatalog::new());
        let transport = Arc::new(ReqwestHttpTransport::new());
        let command_runner = Arc::new(ProcessCommandRunner);
        let provider_registry = default_cli_profiles().into_iter().fold(
            InMemoryProviderRegistry::new(model_catalog.clone())
                .with_ai_provider(Arc::new(OpenAiProvider::new(
                    transport.clone(),
                    model_catalog.clone(),
                )))
                .with_ai_provider(Arc::new(AnthropicProvider::new(
                    transport.clone(),
                    model_catalog.clone(),
                )))
                .with_ai_provider(Arc::new(OllamaProvider::new_cloud(
                    transport.clone(),
                    model_catalog.clone(),
                    &default_ollama_cloud_endpoint(),
                )))
                .with_ai_provider(Arc::new(OllamaProvider::new_local(
                    transport,
                    model_catalog.clone(),
                    &default_ollama_local_endpoint(),
                ))),
            |registry, profile| {
                registry.with_cli_provider(Arc::new(GenericCliProviderExecutor::new(
                    profile,
                    command_runner.clone(),
                )))
            },
        );
        let provider_registry = Arc::new(provider_registry);
        let list_provider_models = Arc::new(ListProviderModelsService::new(model_catalog.clone()));
        let analyze_capture = Arc::new(AnalyzeCaptureService::new(
            provider_registry.clone(),
            repository.clone(),
            repository.clone(),
        ));
        let continue_conversation = Arc::new(ContinueConversationService::new(
            provider_registry.clone(),
            repository.clone(),
            repository.clone(),
        ));
        let ocr_engine = Arc::new(TesseractOcrEngine::new());

        Self {
            snapshot,
            repository,
            model_catalog,
            provider_registry,
            list_provider_models,
            analyze_capture,
            continue_conversation,
            ocr_engine,
            shortcut_bindings: Arc::new(RwLock::new(Vec::new())),
            ocr_language: Arc::new(RwLock::new("por".to_string())),
        }
    }

    pub fn snapshot(&self) -> &BootstrapSnapshot {
        &self.snapshot
    }

    pub fn repository(&self) -> Arc<SqliteAppRepository> {
        self.repository.clone()
    }

    pub fn model_catalog(&self) -> Arc<ManualProviderModelCatalog> {
        self.model_catalog.clone()
    }

    pub fn provider_registry(&self) -> Arc<InMemoryProviderRegistry> {
        self.provider_registry.clone()
    }

    pub fn list_provider_models(&self) -> Arc<ListProviderModelsService> {
        self.list_provider_models.clone()
    }

    pub fn analyze_capture(&self) -> Arc<AnalyzeCaptureService> {
        self.analyze_capture.clone()
    }

    pub fn continue_conversation(&self) -> Arc<ContinueConversationService> {
        self.continue_conversation.clone()
    }

    pub fn ocr_engine(&self) -> Arc<TesseractOcrEngine> {
        self.ocr_engine.clone()
    }

    /// Replace the active shortcut bindings used by the global shortcut handler.
    pub fn apply_shortcuts(&self, bindings: Vec<ShortcutBinding>) {
        if let Ok(mut guard) = self.shortcut_bindings.write() {
            *guard = bindings;
        }
    }

    /// Update the OCR language used by the shortcut-triggered capture path.
    pub fn apply_ocr_language(&self, lang: String) {
        if let Ok(mut guard) = self.ocr_language.write() {
            *guard = lang;
        }
    }

    /// Return the current OCR language code (default "por").
    pub fn current_ocr_language(&self) -> String {
        self.ocr_language
            .read()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_else(|| "por".to_string())
    }

    /// Find the action name for a given accelerator using the same normalization
    /// rules as the global shortcut plugin.
    pub fn find_action_for_accelerator(&self, accel: &str) -> Option<String> {
        let guard = self.shortcut_bindings.read().ok()?;
        find_action_for_bindings(&guard, accel)
    }
}

fn find_action_for_bindings(bindings: &[ShortcutBinding], accel: &str) -> Option<String> {
    let accel_id = parse_shortcut_id(accel)?;

    bindings
        .iter()
        .filter(|binding| binding.enabled)
        .find(|binding| parse_shortcut_id(binding.accelerator.as_str()) == Some(accel_id))
        .map(|binding| binding.action.as_str().to_string())
}

fn parse_shortcut_id(accel: &str) -> Option<u32> {
    accel.parse::<Shortcut>().ok().map(|shortcut| shortcut.id())
}

#[cfg(test)]
mod tests {
    use super::find_action_for_bindings;
    use crate::domain::{
        entities::shortcut_binding::ShortcutBinding,
        value_objects::shortcut::{ShortcutAccelerator, ShortcutAction},
    };
    use tauri_plugin_global_shortcut::Shortcut;

    fn binding(action: &str, accelerator: &str, enabled: bool) -> ShortcutBinding {
        ShortcutBinding {
            action: ShortcutAction::new(action).expect("shortcut action should be valid"),
            accelerator: ShortcutAccelerator::new(accelerator)
                .expect("shortcut accelerator should be valid"),
            enabled,
        }
    }

    #[test]
    fn resolves_action_when_runtime_normalizes_cmd_or_ctrl_accelerator() {
        let bindings = vec![binding("capture_screen", "CmdOrCtrl+Shift+A", true)];
        let runtime_accelerator = "CmdOrCtrl+Shift+A"
            .parse::<Shortcut>()
            .expect("shortcut should parse")
            .into_string();

        let action = find_action_for_bindings(&bindings, &runtime_accelerator);

        assert_eq!(action.as_deref(), Some("capture_screen"));
    }

    #[test]
    fn ignores_disabled_or_invalid_bindings_when_matching_shortcuts() {
        let bindings = vec![
            binding("open_history", "CmdOrCtrl+Shift+H", false),
            ShortcutBinding {
                action: ShortcutAction::new("broken").expect("shortcut action should be valid"),
                accelerator: ShortcutAccelerator::new("CmdOrCtrl+Broken")
                    .expect("value object accepts plus-separated accelerators"),
                enabled: true,
            },
        ];
        let runtime_accelerator = "CmdOrCtrl+Shift+H"
            .parse::<Shortcut>()
            .expect("shortcut should parse")
            .into_string();

        let action = find_action_for_bindings(&bindings, &runtime_accelerator);

        assert_eq!(action, None);
    }
}
