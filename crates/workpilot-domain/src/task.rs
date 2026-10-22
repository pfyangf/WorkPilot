use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::id::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

pub struct Transition {
    pub to: TaskStatus,
    pub now: DateTime<Utc>,
    pub open_child_count: u32,
    pub confirm_parent_done: bool,
}

#[derive(Debug)]
pub struct Task {
    id: EntityId,
    status: TaskStatus,
    priority: Priority,
    is_continuous: bool,
    completed_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    version: u32,
    deleted: bool,
}

impl Task {
    pub fn create(title: &str, now: DateTime<Utc>) -> Result<Self, DomainError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::Validation("标题不能为空".to_string()));
        }
        if title.chars().count() > 200 {
            return Err(DomainError::Validation("标题不能超过 200 个字符".to_string()));
        }
        Ok(Self {
            id: EntityId::generate(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            is_continuous: false,
            completed_at: None,
            updated_at: now,
            version: 1,
            deleted: false,
        })
    }

    /// 非法流转保持原状态。进入 `done` 写入完成时间，离开 `done` 时清空。
    /// 仍有未完成子任务时，必须显式确认，不能静默完成父任务。
    pub fn transition(&mut self, request: Transition) -> Result<(), DomainError> {
        if !allows(self.status, request.to) {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                to: request.to,
            });
        }
        if request.to == TaskStatus::Done
            && request.open_child_count > 0
            && !request.confirm_parent_done
        {
            return Err(DomainError::NeedsConfirmation);
        }
        self.completed_at = if request.to == TaskStatus::Done {
            Some(request.now)
        } else if self.status == TaskStatus::Done {
            None
        } else {
            self.completed_at
        };
        self.status = request.to;
        self.updated_at = request.now;
        self.version += 1;
        Ok(())
    }

    pub fn id(&self) -> &EntityId {
        &self.id
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn is_continuous(&self) -> bool {
        self.is_continuous
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub(crate) fn mark_activity(&mut self, now: DateTime<Utc>) {
        self.updated_at = now;
        self.version += 1;
    }

    pub fn set_continuous(&mut self, value: bool) {
        self.is_continuous = value;
    }
}

fn allows(from: TaskStatus, to: TaskStatus) -> bool {
    use TaskStatus::*;
    matches!(
        (from, to),
        (Todo, InProgress | Blocked | Done | Cancelled)
            | (InProgress, Todo | Blocked | Done | Cancelled)
            | (Blocked, Todo | InProgress | Cancelled)
            | (Done, Todo | InProgress)
            | (Cancelled, Todo)
    )
}
