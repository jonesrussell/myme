//! Repo operations state machine (clone/pull/refresh).
//!
//! Ensures only one operation runs at a time. Used by RepoModel.

/// Operation state for serializing refresh, clone, and pull.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpState {
    #[default]
    Idle,
    BusyRefresh,
    BusyClone(usize),
    BusyPull(usize),
}

impl OpState {
    /// True if a new fetch/refresh can be started.
    pub fn can_start_refresh(self) -> bool {
        matches!(self, OpState::Idle)
    }

    /// True if clone for the given index can be started.
    pub fn can_start_clone(self, _index: usize) -> bool {
        matches!(self, OpState::Idle)
    }

    /// True if pull for the given index can be started.
    pub fn can_start_pull(self, _index: usize) -> bool {
        matches!(self, OpState::Idle)
    }

    /// State after processing RefreshDone message.
    pub fn on_refresh_done(self) -> Self {
        OpState::Idle
    }

    /// State after processing CloneDone message.
    pub fn on_clone_done(self) -> Self {
        OpState::Idle
    }

    /// State after processing PullDone message.
    pub fn on_pull_done(self) -> Self {
        OpState::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_allows_all_ops() {
        let s = OpState::Idle;
        assert!(s.can_start_refresh());
        assert!(s.can_start_clone(0));
        assert!(s.can_start_pull(0));
    }

    #[test]
    fn busy_refresh_blocks_ops() {
        let s = OpState::BusyRefresh;
        assert!(!s.can_start_refresh());
        assert!(!s.can_start_clone(0));
        assert!(!s.can_start_pull(0));
    }

    #[test]
    fn busy_clone_blocks_ops() {
        let s = OpState::BusyClone(1);
        assert!(!s.can_start_refresh());
        assert!(!s.can_start_clone(0));
        assert!(!s.can_start_pull(0));
    }

    #[test]
    fn busy_pull_blocks_ops() {
        let s = OpState::BusyPull(2);
        assert!(!s.can_start_refresh());
        assert!(!s.can_start_clone(0));
        assert!(!s.can_start_pull(0));
    }

    #[test]
    fn refresh_done_transitions_to_idle() {
        assert_eq!(OpState::BusyRefresh.on_refresh_done(), OpState::Idle);
    }

    #[test]
    fn clone_done_transitions_to_idle() {
        assert_eq!(OpState::BusyClone(0).on_clone_done(), OpState::Idle);
    }

    #[test]
    fn pull_done_transitions_to_idle() {
        assert_eq!(OpState::BusyPull(0).on_pull_done(), OpState::Idle);
    }
}
