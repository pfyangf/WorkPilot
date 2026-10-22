use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use workpilot_domain::{
    ContentOrigin, DailyReport, DomainError, Priority, SourceType, Task, TaskStatus, WorkLog,
};

fn rfc3339(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatusDto {
    Todo,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl From<TaskStatus> for TaskStatusDto {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Todo => Self::Todo,
            TaskStatus::InProgress => Self::InProgress,
            TaskStatus::Blocked => Self::Blocked,
            TaskStatus::Done => Self::Done,
            TaskStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityDto {
    Low,
    Medium,
    High,
}

impl From<Priority> for PriorityDto {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => Self::Low,
            Priority::Medium => Self::Medium,
            Priority::High => Self::High,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTypeDto {
    Manual,
    CodingTool,
    System,
    AiGenerated,
}

impl From<SourceType> for SourceTypeDto {
    fn from(value: SourceType) -> Self {
        match value {
            SourceType::Manual => Self::Manual,
            SourceType::CodingTool => Self::CodingTool,
            SourceType::System => Self::System,
            SourceType::AiGenerated => Self::AiGenerated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentOriginDto {
    Generated,
    UserEdited,
}

impl From<ContentOrigin> for ContentOriginDto {
    fn from(value: ContentOrigin) -> Self {
        match value {
            ContentOrigin::Generated => Self::Generated,
            ContentOrigin::UserEdited => Self::UserEdited,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub status: TaskStatusDto,
    pub priority: PriorityDto,
    pub updated_at: String,
    pub version: u32,
    pub deleted: bool,
}

impl From<&Task> for TaskDto {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id().as_str().to_string(),
            status: task.status().into(),
            priority: task.priority().into(),
            updated_at: rfc3339(task.updated_at()),
            version: task.version(),
            deleted: task.deleted(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkLogDto {
    pub task_id: Option<String>,
    pub work_date: String,
    pub content: String,
    pub source_type: SourceTypeDto,
    pub updated_at: String,
    pub version: u32,
    pub deleted: bool,
}

impl From<&WorkLog> for WorkLogDto {
    fn from(log: &WorkLog) -> Self {
        Self {
            task_id: log.task_id().map(|id| id.as_str().to_string()),
            work_date: log.work_date().format("%Y-%m-%d").to_string(),
            content: log.content().to_string(),
            source_type: log.source_type().into(),
            updated_at: rfc3339(log.updated_at()),
            version: log.version(),
            deleted: log.deleted(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyReportDto {
    pub id: String,
    pub work_date: String,
    pub version_no: u32,
    pub content_markdown: String,
    pub content_origin: ContentOriginDto,
    pub updated_at: String,
    pub version: u32,
    pub deleted: bool,
}

impl From<&DailyReport> for DailyReportDto {
    fn from(report: &DailyReport) -> Self {
        Self {
            id: report.id().as_str().to_string(),
            work_date: report.work_date().format("%Y-%m-%d").to_string(),
            version_no: report.version_no(),
            content_markdown: report.content().to_string(),
            content_origin: report.content_origin().into(),
            updated_at: rfc3339(report.updated_at()),
            version: report.version(),
            deleted: report.deleted(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDto {
    pub code: String,
    pub message: String,
}

impl From<&DomainError> for ErrorDto {
    fn from(error: &DomainError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
        }
    }
}

/// 只定义后续同步要用的摘要字段，本阶段不采集、也不上传会话内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodingSessionSnapshot {
    pub id: String,
    pub tool_name: String,
    pub external_session_id: Option<String>,
    pub task_id: Option<String>,
    pub summary: String,
    pub updated_at: String,
    pub version: u32,
    pub deleted: bool,
}

/// 同步元数据占位。本阶段没有网络同步实现。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub device_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub version: u32,
    pub updated_at: String,
    pub deleted: bool,
    pub last_synced_at: Option<String>,
    pub pending: bool,
}
