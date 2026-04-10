use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::configured_provider::{ProviderChannel, ProviderKind},
    value_objects::model_key::ModelKey,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderModel {
    pub provider_kind: ProviderKind,
    pub channel: ProviderChannel,
    pub model_key: ModelKey,
    pub display_name: String,
    pub manually_managed: bool,
    pub is_default: bool,
}

impl ProviderModel {
    pub fn matches_selection(&self, provider_kind: ProviderKind, model_key: &ModelKey) -> bool {
        self.provider_kind == provider_kind && &self.model_key == model_key
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderModel;
    use crate::domain::{
        entities::configured_provider::{ProviderChannel, ProviderKind},
        value_objects::model_key::ModelKey,
    };

    #[test]
    fn matches_selection_by_provider_and_model_key() {
        let selected_model = ModelKey::new("gpt-4.1-mini").expect("model key should be valid");
        let model = ProviderModel {
            provider_kind: ProviderKind::OpenAi,
            channel: ProviderChannel::Http,
            model_key: selected_model.clone(),
            display_name: "GPT-4.1 Mini".to_string(),
            manually_managed: true,
            is_default: true,
        };

        assert!(model.matches_selection(ProviderKind::OpenAi, &selected_model));
    }
}
