use crate::application::errors::ApplicationError;

pub trait TelemetrySink: Send + Sync {
    fn record_info(&self, event: &str, message: &str);

    fn record_warning(&self, event: &str, message: &str);

    fn record_error(&self, event: &str, error: &ApplicationError);
}
