use chrono::{DateTime, NaiveDate, Utc};

use crate::error::DomainError;
use crate::id::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentOrigin {
    Generated,
    UserEdited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportStatus {
    Draft,
    Reviewed,
    Exported,
}

#[derive(Debug)]
pub struct DailyReport {
    id: EntityId,
    work_date: NaiveDate,
    version_no: u32,
    title: String,
    content_markdown: String,
    content_origin: ContentOrigin,
    status: ReportStatus,
    updated_at: DateTime<Utc>,
    version: u32,
    deleted: bool,
}

impl DailyReport {
    pub fn create_generated(
        work_date: NaiveDate,
        title: &str,
        content: &str,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let content = require_content(content)?;
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::Validation("标题不能为空".to_string()));
        }
        Ok(Self {
            id: EntityId::generate(),
            work_date,
            version_no: 1,
            title: title.to_string(),
            content_markdown: content,
            content_origin: ContentOrigin::Generated,
            status: ReportStatus::Draft,
            updated_at: now,
            version: 1,
            deleted: false,
        })
    }

    pub fn edit(&mut self, content: &str, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.content_markdown = require_content(content)?;
        self.content_origin = ContentOrigin::UserEdited;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    /// 已由用户编辑的正文不能被重新生成覆盖。要保留新稿，使用 `fork_generated`。
    pub fn regenerate(&mut self, content: &str, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.content_origin == ContentOrigin::UserEdited {
            return Err(DomainError::ReportEditConflict);
        }
        self.content_markdown = require_content(content)?;
        self.content_origin = ContentOrigin::Generated;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    pub fn fork_generated(
        &self,
        content: &str,
        now: DateTime<Utc>,
    ) -> Result<DailyReport, DomainError> {
        let content = require_content(content)?;
        Ok(DailyReport {
            id: EntityId::generate(),
            work_date: self.work_date,
            version_no: self.version_no + 1,
            title: self.title.clone(),
            content_markdown: content,
            content_origin: ContentOrigin::Generated,
            status: ReportStatus::Draft,
            updated_at: now,
            version: 1,
            deleted: false,
        })
    }

    pub fn id(&self) -> &EntityId {
        &self.id
    }

    pub fn content(&self) -> &str {
        &self.content_markdown
    }

    pub fn content_origin(&self) -> ContentOrigin {
        self.content_origin
    }

    pub fn version_no(&self) -> u32 {
        self.version_no
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn work_date(&self) -> NaiveDate {
        self.work_date
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn status(&self) -> ReportStatus {
        self.status
    }
}

fn require_content(content: &str) -> Result<String, DomainError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(DomainError::Validation("内容不能为空".to_string()));
    }
    Ok(content.to_string())
}
