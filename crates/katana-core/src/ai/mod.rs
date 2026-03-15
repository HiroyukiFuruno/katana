//! AI provider abstraction layer.
//!
//! Defines the traits and types that the rest of the application uses to
//! issue AI requests without knowing about provider-specific authentication,
//! transport, or model details.

use std::collections::HashMap;

/// A normalized AI generation request.
#[derive(Debug, Clone)]
pub struct AiRequest {
    /// Prompt text to send to the provider.
    pub prompt: String,
    /// Optional model identifier (provider interprets or ignores this).
    pub model: Option<String>,
    /// Extra key-value parameters (temperature, max_tokens, etc.).
    pub params: HashMap<String, String>,
}

impl AiRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            params: HashMap::new(),
        }
    }
}

/// A normalized AI generation response.
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// Generated text content from the provider.
    pub content: String,
    /// Provider-specific metadata (model name, token usage, etc.).
    pub metadata: HashMap<String, String>,
}

/// Errors that may arise from AI provider operations.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("No AI provider is configured")]
    NotConfigured,

    #[error("Provider request failed: {0}")]
    RequestFailed(String),

    #[error("Provider returned an invalid response: {0}")]
    InvalidResponse(String),
}

/// The core trait that every AI provider adapter must implement.
///
/// Provider-specific authentication, transport, and retry concerns are
/// entirely encapsulated inside implementations of this trait. The rest of
/// the application never depends on provider-specific types.
pub trait AiProvider: Send + Sync {
    /// A stable identifier for this provider (e.g. "openai", "claude").
    fn id(&self) -> &str;

    /// Human-readable name for display in the UI.
    fn display_name(&self) -> &str;

    /// Execute an AI generation request synchronously.
    ///
    /// Returns `Err(AiError::NotConfigured)` when the provider has no valid
    /// credentials or configuration, so the caller can gracefully disable
    /// AI-dependent commands.
    fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError>;

    /// Whether this provider is ready to serve requests.
    fn is_available(&self) -> bool;
}

/// A provider registry keyed by provider identifier.
///
/// The rest of the application interacts with AI features through the registry
/// rather than through concrete provider types.
#[derive(Default)]
pub struct AiProviderRegistry {
    providers: HashMap<String, Box<dyn AiProvider>>,
    active_id: Option<String>,
}

impl AiProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a provider adapter.
    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        let id = provider.id().to_string();
        self.providers.insert(id, provider);
    }

    /// Activate a registered provider by ID.
    ///
    /// Returns `false` if no provider with that ID is registered.
    pub fn set_active(&mut self, id: &str) -> bool {
        if self.providers.contains_key(id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Execute a request on the active provider.
    ///
    /// Returns `Err(AiError::NotConfigured)` when no provider is active
    /// or when the active provider reports itself unavailable.
    pub fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let id = self.active_id.as_deref().ok_or(AiError::NotConfigured)?;
        // set_active が true を返すのは providers に存在する場合のみ。
        // よって active_id が Some なら providers に必ず存在する。
        let provider = self
            .providers
            .get(id)
            .expect("BUG: active_id is set but provider not found in registry");
        if !provider.is_available() {
            return Err(AiError::NotConfigured);
        }
        provider.execute(request)
    }

    /// Whether the registry has an active, available provider.
    pub fn has_active_provider(&self) -> bool {
        self.active_id
            .as_deref()
            .and_then(|id| self.providers.get(id))
            .map(|p| p.is_available())
            .unwrap_or(false)
    }
}
