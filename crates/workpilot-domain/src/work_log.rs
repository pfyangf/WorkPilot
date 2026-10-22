use chrono::{DateTime, NaiveDate, Utc};

use crate::error::DomainError;
use crate::id::EntityId;
use crate::task::Task;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceType {
    Manual,
    CodingTool,
    System,
    AiGenerated,
}

pub struct WorkLogDraft {
    pub content: String,
    pub work_date: NaiveDate,
    pub source_type: SourceType,
    pub now: DateTime<Utc>,
}

#[derive(Debug)]
pub struct WorkLog {
    task_id: Option<EntityId>,
    work_date: NaiveDate,
    content: String,
    source_type: SourceType,
    updated_at: DateTime<Utc>,
    version: u32,
    deleted: bool,
}

/// 追加一条新日志。不覆盖已有日志，也不改变任务状态。
pub fn append_work_log(
    task: Option<&mut Task>,
    draft: WorkLogDraft,
) -> Result<WorkLog, DomainError> {
    let content = draft.content.trim();
    if content.is_empty() {
        return Err(DomainError::Validation("内容不能为空".to_string()));
    }
    let task_id = task.as_ref().map(|task| task.id().clone());
    if let Some(task) = task {
        task.mark_activity(draft.now);
    }
    Ok(WorkLog {
        task_id,
        work_date: draft.work_date,
        content: content.to_string(),
        source_type: draft.source_type,
        updated_at: draft.now,
        version: 1,
        deleted: false,
    })
}

impl WorkLog {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn task_id(&self) -> Option<EntityId> {
        self.task_id.clone()
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn work_date(&self) -> NaiveDate {
        self.work_date
    }

    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    /// AI 生成内容必须保留来源，不能改记成手工事实。
    pub fn relabel(&mut self, to: SourceType) -> Result<(), DomainError> {
        if self.source_type == SourceType::AiGenerated && to == SourceType::Manual {
            return Err(DomainError::Validation(
                "AI 生成内容不能改为手工记录".to_string(),
            ));
        }
        self.source_type = to;
        Ok(())
    }
}
