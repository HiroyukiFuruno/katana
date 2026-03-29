/* WHY: Defines the traits and types that the rest of the application uses to
     issue AI requests without knowing about provider-specific authentication,
     transport, or model details.
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct AiRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub params: Vec<Param>,
}

impl AiRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            params: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiResponse {
    pub content: String,
    pub metadata: Vec<Param>,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("No AI provider is configured")]
    NotConfigured,

    #[error("Provider request failed: {0}")]
    RequestFailed(String),

    #[error("Provider returned an invalid response: {0}")]
    InvalidResponse(String),
}

/* WHY: Provider-specific authentication, transport, and retry concerns are
     entirely encapsulated inside implementations of this trait.
     The rest of the application never depends on provider-specific types.
*/
pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;

    fn display_name(&self) -> &str;

    fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError>;

    fn is_available(&self) -> bool;
}

/* WHY: The rest of the application interacts with AI features through the registry
     rather than through concrete provider types.
*/
#[derive(Default)]
pub struct AiProviderRegistry {
    providers: Vec<Box<dyn AiProvider>>,
    active_id: Option<String>,
}

impl AiProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        let id = provider.id().to_string();
        if let Some(idx) = self.providers.iter().position(|p| p.id() == id) {
            self.providers[idx] = provider;
        } else {
            self.providers.push(provider);
        }
    }

    pub fn set_active(&mut self, id: &str) -> bool {
        if self.providers.iter().any(|p| p.id() == id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let id = self.active_id.as_deref().ok_or(AiError::NotConfigured)?;
        /* WHY: `set_active` returns `true` only if it exists in providers.
               Therefore, if `active_id` is `Some`, it must exist in providers.
        */
        let provider = self
            .providers
            .iter()
            .find(|p| p.id() == id)
            .expect("BUG: active_id is set but provider not found in registry");
        if !provider.is_available() {
            return Err(AiError::NotConfigured);
        }
        provider.execute(request)
    }

    pub fn has_active_provider(&self) -> bool {
        self.active_id
            .as_deref()
            .and_then(|id| self.providers.iter().find(|p| p.id() == id))
            .map(|p| p.is_available())
            .unwrap_or(false)
    }
}
