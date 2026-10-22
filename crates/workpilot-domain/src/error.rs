use crate::task::TaskStatus;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("{0}")]
    Validation(String),
    #[error("不能从 {from} 变为 {to}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },
    #[error("仍有未完成子任务，需要确认")]
    NeedsConfirmation,
    #[error("日报已编辑，不能覆盖正文")]
    ReportEditConflict,
    #[error("不存在")]
    NotFound,
}

impl DomainError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::InvalidTransition { .. } => "invalid_transition",
            Self::NeedsConfirmation => "needs_confirmation",
            Self::ReportEditConflict => "report_edit_conflict",
            Self::NotFound => "not_found",
        }
    }
}
