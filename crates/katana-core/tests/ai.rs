use katana_core::ai::*;
use std::collections::HashMap;

struct DisabledProvider;

impl AiProvider for DisabledProvider {
    fn id(&self) -> &str {
        "disabled"
    }
    fn display_name(&self) -> &str {
        "Disabled"
    }
    fn is_available(&self) -> bool {
        false
    }
    fn execute(&self, _: &AiRequest) -> Result<AiResponse, AiError> {
        Err(AiError::NotConfigured)
    }
}

struct EchoProvider;

impl AiProvider for EchoProvider {
    fn id(&self) -> &str {
        "echo"
    }
    fn display_name(&self) -> &str {
        "Echo"
    }
    fn is_available(&self) -> bool {
        true
    }
    fn execute(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        Ok(AiResponse {
            content: req.prompt.clone(),
            metadata: HashMap::new(),
        })
    }
}

#[test]
fn registry_returns_not_configured_when_empty() {
    let registry = AiProviderRegistry::new();
    let req = AiRequest::new("hello");
    assert!(matches!(
        registry.execute(&req),
        Err(AiError::NotConfigured)
    ));
}

#[test]
fn registry_with_disabled_provider_returns_not_configured() {
    let mut registry = AiProviderRegistry::new();
    registry.register(Box::new(DisabledProvider));
    registry.set_active("disabled");
    let req = AiRequest::new("hello");
    assert!(matches!(
        registry.execute(&req),
        Err(AiError::NotConfigured)
    ));
}

#[test]
fn registry_with_available_provider_executes() {
    let mut registry = AiProviderRegistry::new();
    registry.register(Box::new(EchoProvider));
    registry.set_active("echo");
    let req = AiRequest::new("test prompt");
    let resp = registry.execute(&req).unwrap();
    assert_eq!(resp.content, "test prompt");
}

#[test]
fn has_active_provider_reflects_available_state() {
    let mut registry = AiProviderRegistry::new();
    assert!(!registry.has_active_provider());
    registry.register(Box::new(DisabledProvider));
    registry.set_active("disabled");
    assert!(!registry.has_active_provider());
    registry.register(Box::new(EchoProvider));
    registry.set_active("echo");
    assert!(registry.has_active_provider());
}
