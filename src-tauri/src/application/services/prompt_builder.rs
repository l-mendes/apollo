use crate::application::dto::analysis::AnalyzeCaptureRequest;
use crate::domain::entities::conversation_message::ConversationMessage;

#[derive(Debug, Default, Clone)]
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn compose_analysis_prompt(&self, request: &AnalyzeCaptureRequest) -> String {
        let mut sections = vec![
            format!("System instruction:\n{}", request.base_prompt.trim()),
            format!("OCR text:\n{}", request.ocr_text.trim()),
        ];

        if let Some(user_notes) = request
            .user_notes
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            sections.push(format!("User notes:\n{}", user_notes.trim()));
        }

        if !request.conversation_context.is_empty() {
            sections.push(format!(
                "Conversation context:\n{}",
                Self::render_conversation(&request.conversation_context)
            ));
        }

        sections.join("\n\n")
    }

    pub fn compose_follow_up_prompt(
        &self,
        existing_messages: &[ConversationMessage],
        prompt: &str,
    ) -> String {
        let mut sections = Vec::new();

        if !existing_messages.is_empty() {
            sections.push(format!(
                "Conversation context:\n{}",
                Self::render_conversation(existing_messages)
            ));
        }

        sections.push(format!("Follow-up request:\n{}", prompt.trim()));

        sections.join("\n\n")
    }

    pub fn compose_user_turn(&self, request: &AnalyzeCaptureRequest) -> String {
        match request
            .user_notes
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            Some(user_notes) => format!(
                "Captured text: {}\nUser notes: {}",
                request.ocr_text.trim(),
                user_notes.trim()
            ),
            None => format!("Captured text: {}", request.ocr_text.trim()),
        }
    }

    fn render_conversation(messages: &[ConversationMessage]) -> String {
        messages
            .iter()
            .map(|message| format!("{}: {}", message.role.as_str(), message.content.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
