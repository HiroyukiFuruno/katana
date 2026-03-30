use katana_core::update::ReleaseInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePhase {
    Downloading { progress: f32 },
    Installing { progress: f32 },
    ReadyToRelaunch,
}

pub struct UpdateState {
    pub available: Option<ReleaseInfo>,
    pub checking: bool,
    pub phase: Option<UpdatePhase>,
    pub check_error: Option<String>,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateState {
    pub fn new() -> Self {
        Self {
            available: None,
            checking: false,
            phase: None,
            check_error: None,
        }
    }
}