pub mod download;
pub mod installer;
pub mod version;

pub use download::*;
pub use installer::*;
pub use version::*;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProgress {
    Downloading { downloaded: u64, total: Option<u64> },
    Extracting { current: usize, total: usize },
}

#[derive(Debug, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpdateAvailable(ReleaseInfo),
    Downloading,
    ReadyToRestart(UpdatePreparation),
    Error(String),
}

pub struct UpdateManager {
    pub current_version: String,
    pub api_url_override: Option<String>,
    pub target_app_path: std::path::PathBuf,
    pub state: UpdateState,
    pub last_checked: Option<std::time::Instant>,
    pub check_interval: std::time::Duration,
}

impl UpdateManager {
    pub fn new(current_version: String, target_app_path: std::path::PathBuf) -> Self {
        const DEFAULT_CHECK_INTERVAL_SECS: u64 = 86_400;
        Self {
            current_version,
            api_url_override: None,
            target_app_path,
            state: UpdateState::Idle,
            last_checked: None,
            check_interval: std::time::Duration::from_secs(DEFAULT_CHECK_INTERVAL_SECS),
        }
    }

    pub fn should_check_for_updates(&self) -> bool {
        match self.last_checked {
            Some(last) => last.elapsed() >= self.check_interval,
            None => true,
        }
    }

    pub fn set_api_url_override(&mut self, url: String) {
        self.api_url_override = Some(url);
    }

    pub fn set_check_interval(&mut self, interval: std::time::Duration) {
        self.check_interval = interval;
    }

    pub fn transition_to(&mut self, new_state: UpdateState) {
        if matches!(new_state, UpdateState::Checking) {
            self.last_checked = Some(std::time::Instant::now());
        }
        self.state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_manager_and_state() {
        let target = std::path::PathBuf::from("/Applications/KatanA.app");
        let mut manager = UpdateManager::new("0.6.4".to_string(), target.clone());

        assert_eq!(manager.current_version, "0.6.4");
        assert_eq!(manager.target_app_path, target);
        assert!(matches!(manager.state, UpdateState::Idle));

        assert!(manager.should_check_for_updates());

        manager.set_api_url_override("http://localhost".to_string());
        assert_eq!(
            manager.api_url_override.as_deref(),
            Some("http://localhost")
        );

        manager.set_check_interval(std::time::Duration::from_secs(3600));
        assert_eq!(manager.check_interval, std::time::Duration::from_secs(3600));

        manager.transition_to(UpdateState::Checking);
        assert!(matches!(manager.state, UpdateState::Checking));
        assert!(manager.last_checked.is_some());

        assert!(!manager.should_check_for_updates());

        manager.last_checked =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(4000));
        assert!(manager.should_check_for_updates());

        manager.transition_to(UpdateState::Error("dummy error".to_string()));
        assert!(matches!(manager.state, UpdateState::Error(_)));

        let default_state = UpdateState::default();
        assert!(matches!(default_state, UpdateState::Idle));
    }
}
