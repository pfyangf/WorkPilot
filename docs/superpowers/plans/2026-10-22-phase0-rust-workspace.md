# Phase 0 Rust Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立一个可在 Windows 上测试的 Rust workspace，证明任务流转、日志追加和已编辑日报保护这些规则不依赖 UI 与数据库。

**Architecture:** `workpilot-domain` 只包含实体、状态机和领域错误。`workpilot-contracts` 把这些实体转成可序列化的 DTO。两个 crate 都不访问文件、网络和平台 API。

**Tech Stack:** Rust stable、Cargo edition 2021、serde、serde_json、thiserror 1、uuid 1（v7）、chrono 0.4（不启用 clock）。

## Global Constraints

- Rust stable toolchain，Cargo edition 2021。
- Domain 不调用数据库、HTTP、Tauri、JNI、NAPI 或操作系统 API。
- 稳定标识使用 UUID v7 字符串，不使用自增整数。
- 时间使用 UTC；`work_date` 使用 `YYYY-MM-DD`。
- 用户编辑过的日报重新生成时返回 `report_edit_conflict`，正文保持不变。
- Phase 0 不创建 Windows、Android、HarmonyOS 应用，不创建 SQLite，不调用模型，不实现同步。
- 错误码使用 `validation`、`invalid_transition`、`needs_confirmation`、`report_edit_conflict`、`not_found`。
- 未实际执行的命令不得写成通过。JNI、NAPI、Tauri 在兼容性记录中写明未验证。
- 未获用户明确要求时不创建 git commit。
- 不引入 Python、PySide6、SQLAlchemy 或 Alembic。

---

### Task 1: Workspace 与任务状态流转

**Files:**

- Create: `.gitignore`
- Create: `rust-toolchain.toml`
- Create: `Cargo.toml`
- Create: `crates/workpilot-domain/Cargo.toml`
- Create: `crates/workpilot-domain/src/lib.rs`
- Create: `crates/workpilot-domain/src/error.rs`
- Create: `crates/workpilot-domain/src/id.rs`
- Create: `crates/workpilot-domain/src/task.rs`
- Test: `crates/workpilot-domain/tests/task.rs`

**Interfaces:**

- Consumes: 无。
- Produces:
  - `DomainError::code(&self) -> &'static str`
  - `EntityId::generate() -> EntityId`
  - `EntityId::as_str(&self) -> &str`
  - `EntityId::is_uuid_v7(&self) -> bool`
  - `TaskStatus`：`Todo`、`InProgress`、`Blocked`、`Done`、`Cancelled`；`as_str()` 返回 `todo`、`in_progress`、`blocked`、`done`、`cancelled`
  - `Priority`：`Low`、`Medium`、`High`
  - `Transition { to: TaskStatus, now: DateTime<Utc>, open_child_count: u32, confirm_parent_done: bool }`
  - `Task::create(title: &str, now: DateTime<Utc>) -> Result<Task, DomainError>`
  - `Task::transition(&mut self, request: Transition) -> Result<(), DomainError>`
  - `Task::id/status/priority/is_continuous/completed_at/updated_at/version/deleted`

- [ ] **Step 1: 写会失败的任务测试**

创建 `.gitignore`：

```gitignore
/target
```

创建 `rust-toolchain.toml`：

```toml
[toolchain]
channel = "stable"
```

创建 `Cargo.toml`：

```toml
[workspace]
resolver = "2"
members = ["crates/workpilot-domain"]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
thiserror = "1"
uuid = { version = "1", features = ["v7"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
```

创建 `crates/workpilot-domain/Cargo.toml`：

```toml
[package]
name = "workpilot-domain"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror.workspace = true
uuid.workspace = true
chrono.workspace = true
```

创建 `crates/workpilot-domain/src/id.rs`：

```rust
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(String);

impl EntityId {
    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_uuid_v7(&self) -> bool {
        Uuid::parse_str(&self.0)
            .ok()
            .and_then(|id| id.get_version())
            == Some(uuid::Version::SortRand)
    }
}
```

创建 `crates/workpilot-domain/src/lib.rs`。此步故意还不导出 `Task`，用来确认测试先失败：

```rust
mod id;

pub use id::EntityId;
```

创建 `crates/workpilot-domain/tests/task.rs`：

```rust
use chrono::{DateTime, TimeZone, Utc};
use workpilot_domain::{Task, TaskStatus, Transition};

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 22, hour, 0, 0).unwrap()
}

#[test]
fn create_task_uses_uuid_v7_and_starts_as_todo() {
    let task = Task::create("编写日报", at(9)).unwrap();
    assert!(task.id().is_uuid_v7());
    assert_eq!(task.status(), TaskStatus::Todo);
    assert_eq!(task.priority(), workpilot_domain::Priority::Medium);
    assert!(!task.is_continuous());
    assert_eq!(task.completed_at(), None);
    assert_eq!(task.version(), 1);
    assert!(!task.deleted());
    assert_eq!(task.updated_at(), at(9));
}

#[test]
fn reject_blank_or_too_long_title() {
    let blank = Task::create("   ", at(9)).unwrap_err();
    assert_eq!(blank.code(), "validation");
    let long = "题".repeat(201);
    let too_long = Task::create(&long, at(9)).unwrap_err();
    assert_eq!(too_long.code(), "validation");
    assert!(Task::create(&"题".repeat(200), at(9)).is_ok());
}

#[test]
fn legal_transitions_set_and_clear_completed_at() {
    let mut task = Task::create("修复缺陷", at(9)).unwrap();
    task.transition(Transition {
        to: TaskStatus::InProgress,
        now: at(10),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert_eq!(task.completed_at(), None);
    assert_eq!(task.version(), 2);
    assert_eq!(task.updated_at(), at(10));

    task.transition(Transition {
        to: TaskStatus::Done,
        now: at(11),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    assert_eq!(task.status(), TaskStatus::Done);
    assert_eq!(task.completed_at(), Some(at(11)));

    task.transition(Transition {
        to: TaskStatus::InProgress,
        now: at(12),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert_eq!(task.completed_at(), None);
}

#[test]
fn illegal_transition_keeps_the_original_status() {
    let mut task = Task::create("等待资料", at(9)).unwrap();
    task.transition(Transition {
        to: TaskStatus::Blocked,
        now: at(10),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    let error = task
        .transition(Transition {
            to: TaskStatus::Done,
            now: at(11),
            open_child_count: 0,
            confirm_parent_done: false,
        })
        .unwrap_err();
    assert_eq!(error.code(), "invalid_transition");
    assert_eq!(task.status(), TaskStatus::Blocked);
    assert_eq!(task.completed_at(), None);
    assert_eq!(task.version(), 2);

    let same = task
        .transition(Transition {
            to: TaskStatus::Blocked,
            now: at(12),
            open_child_count: 0,
            confirm_parent_done: false,
        })
        .unwrap_err();
    assert_eq!(same.code(), "invalid_transition");
}

#[test]
fn cancelled_task_can_return_to_todo() {
    let mut task = Task::create("取消的任务", at(9)).unwrap();
    task.transition(Transition {
        to: TaskStatus::Cancelled,
        now: at(10),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    task.transition(Transition {
        to: TaskStatus::Todo,
        now: at(11),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    assert_eq!(task.status(), TaskStatus::Todo);
    assert_eq!(task.completed_at(), None);
}

#[test]
fn not_found_has_stable_code() {
    assert_eq!(workpilot_domain::DomainError::NotFound.code(), "not_found");
}
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cargo test -p workpilot-domain --test task`

Expected: 编译失败，错误包含 `cannot find type Task` 或 unresolved import `Task`。不能把这次失败写成测试通过。

- [ ] **Step 3: 实现任务模型**

创建 `crates/workpilot-domain/src/error.rs`：

```rust
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
```

把 `crates/workpilot-domain/src/lib.rs` 替换为：

```rust
mod error;
mod id;
mod task;

pub use error::DomainError;
pub use id::EntityId;
pub use task::{Priority, Task, TaskStatus, Transition};
```

创建 `crates/workpilot-domain/src/task.rs`：

```rust
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

    pub fn transition(&mut self, request: Transition) -> Result<(), DomainError> {
        let _ = (request.open_child_count, request.confirm_parent_done);
        if !allows(self.status, request.to) {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                to: request.to,
            });
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

    pub(crate) fn set_continuous(&mut self, value: bool) {
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
```

`error.rs` 引用 `TaskStatus`，因此 `task` 模块必须在这一步加入。`mark_activity` 和 `set_continuous` 供后续日志任务使用。

- [ ] **Step 4: 运行测试并确认通过**

Run: `cargo test -p workpilot-domain --test task`

Expected: 6 个测试全部通过，进程退出码为 0。

- [ ] **Step 5: 保留变更**

不执行 git commit。向用户展示本任务新增的文件。

---

### Task 2: 父任务仍有未完成子任务时要求确认

**Files:**

- Modify: `crates/workpilot-domain/src/task.rs`
- Test: `crates/workpilot-domain/tests/task.rs`

**Interfaces:**

- Consumes: Task 1 的 `Task::transition` 与 `Transition`。
- Produces: 相同签名。`to == Done` 且 `open_child_count > 0` 且 `confirm_parent_done == false` 时返回 `DomainError::NeedsConfirmation`，不修改任务。确认后才进入 `done` 并写入 `completed_at`。

- [ ] **Step 1: 写会失败的确认测试**

在 `crates/workpilot-domain/tests/task.rs` 末尾追加：

```rust
#[test]
fn parent_done_requires_confirmation_while_children_are_open() {
    let mut task = Task::create("父任务", at(9)).unwrap();
    task.transition(Transition {
        to: TaskStatus::InProgress,
        now: at(10),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    let error = task
        .transition(Transition {
            to: TaskStatus::Done,
            now: at(11),
            open_child_count: 2,
            confirm_parent_done: false,
        })
        .unwrap_err();
    assert_eq!(error.code(), "needs_confirmation");
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert_eq!(task.completed_at(), None);
    assert_eq!(task.version(), 2);

    task.transition(Transition {
        to: TaskStatus::Done,
        now: at(12),
        open_child_count: 2,
        confirm_parent_done: true,
    })
    .unwrap();
    assert_eq!(task.status(), TaskStatus::Done);
    assert_eq!(task.completed_at(), Some(at(12)));
}

#[test]
fn blocked_parent_cannot_skip_to_done_by_confirming() {
    let mut task = Task::create("受阻父任务", at(9)).unwrap();
    task.transition(Transition {
        to: TaskStatus::Blocked,
        now: at(10),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();
    let error = task
        .transition(Transition {
            to: TaskStatus::Done,
            now: at(11),
            open_child_count: 1,
            confirm_parent_done: true,
        })
        .unwrap_err();
    assert_eq!(error.code(), "invalid_transition");
    assert_eq!(task.status(), TaskStatus::Blocked);
}
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cargo test -p workpilot-domain --test task parent_done_requires_confirmation_while_children_are_open`

Expected: 失败。`unwrap_err` 作用在 `Ok` 上，因为当前实现忽略了未完成子任务。

- [ ] **Step 3: 在状态变更前检查子任务**

把 `Task::transition` 替换为：

```rust
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
```

- [ ] **Step 4: 运行测试并确认通过**

Run: `cargo test -p workpilot-domain --test task`

Expected: Task 1 与 Task 2 的测试全部通过，退出码为 0。

- [ ] **Step 5: 保留变更**

不执行 git commit。

---

### Task 3: 工作日志只追加且不改变任务状态

**Files:**

- Create: `crates/workpilot-domain/src/work_log.rs`
- Modify: `crates/workpilot-domain/src/lib.rs`
- Test: `crates/workpilot-domain/tests/work_log.rs`

**Interfaces:**

- Consumes: `Task::mark_activity`、`Task::set_continuous`、`Task::status`、`EntityId`、`DomainError`。
- Produces:
  - `SourceType`：`Manual`、`CodingTool`、`System`、`AiGenerated`
  - `WorkLogDraft { content: String, work_date: NaiveDate, source_type: SourceType, now: DateTime<Utc> }`
  - `append_work_log(task: Option<&mut Task>, draft: WorkLogDraft) -> Result<WorkLog, DomainError>`
  - `WorkLog::content/task_id/work_date/source_type/updated_at/version/deleted`
  - `WorkLog::relabel(&mut self, to: SourceType) -> Result<(), DomainError>`

- [ ] **Step 1: 写会失败的日志测试**

创建 `crates/workpilot-domain/tests/work_log.rs`：

```rust
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use workpilot_domain::{
    append_work_log, SourceType, Task, TaskStatus, Transition, WorkLogDraft,
};

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 22, hour, 0, 0).unwrap()
}

fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
}

#[test]
fn appending_logs_does_not_change_task_status_or_old_content() {
    let mut task = Task::create("持续跟进", at(9)).unwrap();
    task.set_continuous(true);
    task.transition(Transition {
        to: TaskStatus::InProgress,
        now: at(9),
        open_child_count: 0,
        confirm_parent_done: false,
    })
    .unwrap();

    let first = append_work_log(
        Some(&mut task),
        WorkLogDraft {
            content: "  完成接口草稿  ".to_string(),
            work_date: day(22),
            source_type: SourceType::Manual,
            now: at(10),
        },
    )
    .unwrap();
    let second = append_work_log(
        Some(&mut task),
        WorkLogDraft {
            content: "补充边界说明".to_string(),
            work_date: day(23),
            source_type: SourceType::Manual,
            now: at(11),
        },
    )
    .unwrap();

    assert_eq!(first.content(), "完成接口草稿");
    assert_eq!(second.content(), "补充边界说明");
    assert_eq!(first.task_id(), Some(task.id().clone()));
    assert_eq!(first.work_date(), day(22));
    assert_eq!(second.work_date(), day(23));
    assert_eq!(first.version(), 1);
    assert!(!first.deleted());
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert!(task.is_continuous());
    assert_eq!(task.updated_at(), at(11));
    assert_eq!(task.version(), 3);
}

#[test]
fn blank_log_content_is_rejected() {
    let error = append_work_log(
        None,
        WorkLogDraft {
            content: "  ".to_string(),
            work_date: day(22),
            source_type: SourceType::Manual,
            now: at(10),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "validation");
}

#[test]
fn ai_generated_log_cannot_be_relabeled_as_manual() {
    let mut log = append_work_log(
        None,
        WorkLogDraft {
            content: "模型草稿".to_string(),
            work_date: day(22),
            source_type: SourceType::AiGenerated,
            now: at(10),
        },
    )
    .unwrap();
    let error = log.relabel(SourceType::Manual).unwrap_err();
    assert_eq!(error.code(), "validation");
    assert_eq!(log.source_type(), SourceType::AiGenerated);
}
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cargo test -p workpilot-domain --test work_log`

Expected: 编译失败，找不到 `append_work_log`。

- [ ] **Step 3: 实现追加日志**

创建 `crates/workpilot-domain/src/work_log.rs`：

```rust
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

pub struct WorkLog {
    task_id: Option<EntityId>,
    work_date: NaiveDate,
    content: String,
    source_type: SourceType,
    updated_at: DateTime<Utc>,
    version: u32,
    deleted: bool,
}

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
```

把 `crates/workpilot-domain/src/lib.rs` 替换为：

```rust
mod error;
mod id;
mod task;
mod work_log;

pub use error::DomainError;
pub use id::EntityId;
pub use task::{Priority, Task, TaskStatus, Transition};
pub use work_log::{append_work_log, SourceType, WorkLog, WorkLogDraft};
```

`Task::set_continuous` 在 Task 1 中是 `pub(crate)`。集成测试位于另一个 crate，必须改成 `pub`：

```rust
pub fn set_continuous(&mut self, value: bool) {
    self.is_continuous = value;
}
```

- [ ] **Step 4: 运行测试并确认通过**

Run: `cargo test -p workpilot-domain --test work_log`

Expected: 3 个测试通过，退出码为 0。再运行 `cargo test -p workpilot-domain`，原有任务测试仍然通过。

- [ ] **Step 5: 保留变更**

不执行 git commit。

---

### Task 4: 已编辑日报不能被重新生成覆盖

**Files:**

- Create: `crates/workpilot-domain/src/report.rs`
- Modify: `crates/workpilot-domain/src/lib.rs`
- Test: `crates/workpilot-domain/tests/report.rs`

**Interfaces:**

- Consumes: `EntityId`、`DomainError`。
- Produces:
  - `ContentOrigin`：`Generated`、`UserEdited`
  - `ReportStatus`：`Draft`、`Reviewed`、`Exported`
  - `DailyReport::create_generated(work_date, title, content, now) -> Result<DailyReport, DomainError>`
  - `DailyReport::edit(&mut self, content, now) -> Result<(), DomainError>`
  - `DailyReport::regenerate(&mut self, content, now) -> Result<(), DomainError>`
  - `DailyReport::fork_generated(&self, content, now) -> Result<DailyReport, DomainError>`
  - `DailyReport::content/content_origin/version_no/version/work_date`

- [ ] **Step 1: 写会失败的日报测试**

创建 `crates/workpilot-domain/tests/report.rs`：

```rust
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use workpilot_domain::{ContentOrigin, DailyReport};

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 22, hour, 0, 0).unwrap()
}

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 22).unwrap()
}

#[test]
fn regenerate_replaces_generated_content() {
    let mut report = DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
    report.regenerate("根据日志重写", at(10)).unwrap();
    assert_eq!(report.content(), "根据日志重写");
    assert_eq!(report.content_origin(), ContentOrigin::Generated);
    assert_eq!(report.version_no(), 1);
    assert_eq!(report.version(), 2);
}

#[test]
fn regenerate_does_not_overwrite_user_edits() {
    let mut report = DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
    report.edit("用户改过的正文", at(10)).unwrap();
    let version_before = report.version();
    let error = report.regenerate("模型新稿", at(11)).unwrap_err();
    assert_eq!(error.code(), "report_edit_conflict");
    assert_eq!(report.content(), "用户改过的正文");
    assert_eq!(report.content_origin(), ContentOrigin::UserEdited);
    assert_eq!(report.version(), version_before);
}

#[test]
fn fork_keeps_the_edited_report_and_creates_a_new_version() {
    let mut report = DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
    report.edit("用户改过的正文", at(10)).unwrap();
    let forked = report.fork_generated("另一份生成稿", at(11)).unwrap();
    assert_eq!(report.content(), "用户改过的正文");
    assert_eq!(report.version_no(), 1);
    assert_eq!(forked.content(), "另一份生成稿");
    assert_eq!(forked.content_origin(), ContentOrigin::Generated);
    assert_eq!(forked.version_no(), 2);
    assert_eq!(forked.version(), 1);
    assert_eq!(forked.work_date(), today());
    assert_ne!(forked.id().as_str(), report.id().as_str());
}
```

`DailyReport` 需要公开 `id()`。实现时加上。

- [ ] **Step 2: 运行测试并确认失败**

Run: `cargo test -p workpilot-domain --test report`

Expected: 编译失败，找不到 `DailyReport`。

- [ ] **Step 3: 实现日报版本规则**

创建 `crates/workpilot-domain/src/report.rs`：

```rust
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
}

fn require_content(content: &str) -> Result<String, DomainError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(DomainError::Validation("内容不能为空".to_string()));
    }
    Ok(content.to_string())
}
```

把 `crates/workpilot-domain/src/lib.rs` 替换为：

```rust
mod error;
mod id;
mod report;
mod task;
mod work_log;

pub use error::DomainError;
pub use id::EntityId;
pub use report::{ContentOrigin, DailyReport, ReportStatus};
pub use task::{Priority, Task, TaskStatus, Transition};
pub use work_log::{append_work_log, SourceType, WorkLog, WorkLogDraft};
```

- [ ] **Step 4: 运行测试并确认通过**

Run: `cargo test -p workpilot-domain --test report`

Expected: 3 个测试通过。再运行 `cargo test -p workpilot-domain`，全部通过。

- [ ] **Step 5: 保留变更**

不执行 git commit。

---

### Task 5: 跨端 DTO 与稳定错误码

**Files:**

- Modify: `Cargo.toml`
- Create: `crates/workpilot-contracts/Cargo.toml`
- Create: `crates/workpilot-contracts/src/lib.rs`
- Test: `crates/workpilot-contracts/tests/dto.rs`

**Interfaces:**

- Consumes: `Task`、`WorkLog`、`DailyReport`、`DomainError`。
- Produces:
  - `TaskDto`、`WorkLogDto`、`DailyReportDto`、`ErrorDto`
  - `CodingSessionSnapshot`、`SyncMetadata`
  - `From<&Task> for TaskDto`、`From<&WorkLog> for WorkLogDto`、`From<&DailyReport> for DailyReportDto`、`From<&DomainError> for ErrorDto`
  - JSON 字段使用 snake_case。状态值为 `todo`、`in_progress`、`blocked`、`done`、`cancelled`。

- [ ] **Step 1: 写会失败的契约测试**

在根 `Cargo.toml` 的 `members` 中加入 contracts，并加入 serde：

```toml
[workspace]
resolver = "2"
members = ["crates/workpilot-domain", "crates/workpilot-contracts"]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
uuid = { version = "1", features = ["v7"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
```

创建 `crates/workpilot-contracts/Cargo.toml`：

```toml
[package]
name = "workpilot-contracts"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
workpilot-domain = { path = "../workpilot-domain" }
```

创建 `crates/workpilot-contracts/src/lib.rs`，此步只留空模块说明，让测试无法找到类型：

```rust
```

空文件会导致测试编译失败。然后创建 `crates/workpilot-contracts/tests/dto.rs`：

```rust
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use workpilot_contracts::{
    CodingSessionSnapshot, DailyReportDto, ErrorDto, SyncMetadata, TaskDto, WorkLogDto,
};
use workpilot_domain::{
    append_work_log, DailyReport, DomainError, SourceType, Task, WorkLogDraft,
};

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 22, hour, 0, 0).unwrap()
}

#[test]
fn task_dto_round_trips_sync_fields() {
    let task = Task::create("共享任务", at(9)).unwrap();
    let dto = TaskDto::from(&task);
    let json = serde_json::to_value(&dto).unwrap();
    assert_eq!(json["id"], task.id().as_str());
    assert_eq!(json["status"], "todo");
    assert_eq!(json["priority"], "medium");
    assert_eq!(json["version"], 1);
    assert_eq!(json["deleted"], false);
    assert!(json["updated_at"].as_str().unwrap().contains("2026-10-22"));
    let restored: TaskDto = serde_json::from_value(json).unwrap();
    assert_eq!(restored, dto);
}

#[test]
fn work_log_and_report_dtos_keep_dates_and_origin() {
    let log = append_work_log(
        None,
        WorkLogDraft {
            content: "记录进展".to_string(),
            work_date: NaiveDate::from_ymd_opt(2026, 10, 22).unwrap(),
            source_type: SourceType::Manual,
            now: at(10),
        },
    )
    .unwrap();
    let log_dto = WorkLogDto::from(&log);
    let log_json = serde_json::to_value(&log_dto).unwrap();
    assert_eq!(log_json["work_date"], "2026-10-22");
    assert_eq!(log_json["source_type"], "manual");
    assert_eq!(log_json["version"], 1);
    assert_eq!(log_json["deleted"], false);

    let report = DailyReport::create_generated(
        NaiveDate::from_ymd_opt(2026, 10, 22).unwrap(),
        "2026-10-22 日报",
        "正文",
        at(11),
    )
    .unwrap();
    let report_dto = DailyReportDto::from(&report);
    let report_json = serde_json::to_value(&report_dto).unwrap();
    assert_eq!(report_json["content_origin"], "generated");
    assert_eq!(report_json["version_no"], 1);
    assert_eq!(report_json["deleted"], false);
}

#[test]
fn domain_error_uses_stable_code() {
    let dto = ErrorDto::from(&DomainError::ReportEditConflict);
    assert_eq!(dto.code, "report_edit_conflict");
    let json = serde_json::to_string(&dto).unwrap();
    assert!(json.contains("report_edit_conflict"));
}

#[test]
fn deferred_sync_types_round_trip() {
    let snapshot = CodingSessionSnapshot {
        id: "0190".to_string(),
        tool_name: "example".to_string(),
        external_session_id: None,
        task_id: None,
        summary: "完成了草稿".to_string(),
        updated_at: "2026-10-22T01:00:00Z".to_string(),
        version: 1,
        deleted: false,
    };
    let metadata = SyncMetadata {
        device_id: "device-1".to_string(),
        entity_type: "task".to_string(),
        entity_id: snapshot.id.clone(),
        version: 1,
        updated_at: snapshot.updated_at.clone(),
        deleted: false,
        last_synced_at: None,
        pending: true,
    };
    let snapshot_restored: CodingSessionSnapshot =
        serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap();
    let metadata_restored: SyncMetadata =
        serde_json::from_str(&serde_json::to_string(&metadata).unwrap()).unwrap();
    assert_eq!(snapshot_restored, snapshot);
    assert_eq!(metadata_restored, metadata);
    assert!(metadata_restored.pending);
}
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cargo test -p workpilot-contracts --test dto`

Expected: 编译失败，找不到 `TaskDto`。

- [ ] **Step 3: 实现 DTO**

把 `crates/workpilot-contracts/src/lib.rs` 替换为下面这一份完整文件：

```rust
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
```

- [ ] **Step 4: 运行测试并确认通过**

Run: `cargo test -p workpilot-contracts --test dto`

Expected: 4 个测试通过。再运行 `cargo test --workspace`，domain 与 contracts 全部通过。

- [ ] **Step 5: 保留变更**

不执行 git commit。

---

### Task 6: 记录实际构建结果

**Files:**

- Create: `README.md`
- Create: `docs/compatibility/phase0.md`
- Modify: `docs/README_AI_CODING.md`

**Interfaces:**

- Consumes: Task 1 到 Task 5 的 crate。
- Produces: 根 README 中的构建命令，以及只包含本次命令输出的兼容性记录。

- [ ] **Step 1: 执行命令并保存原始输出**

在仓库根目录依次运行：

```powershell
rustc --version
cargo --version
cargo test --workspace
```

记录每条命令的标准输出、标准错误和退出码。测试失败时先修复，再重新运行；兼容性记录只保留最后一次完整结果。

- [ ] **Step 2: 写 README**

创建 `README.md`：

~~~~markdown
# WorkPilot

共享 Rust 业务核心。Windows、Android 和 HarmonyOS 的界面尚未创建。

## 要求

- Rust stable
- 本阶段不需要 Node、Android SDK、DevEco Studio 或模型密钥

## 测试

```powershell
cargo test -p workpilot-domain
cargo test -p workpilot-contracts
cargo test --workspace
```

Phase 0 的实际结果见 `docs/compatibility/phase0.md`。
~~~~

- [ ] **Step 3: 写兼容性记录**

创建 `docs/compatibility/phase0.md`。版本号和退出码必须来自 Step 1 的输出，按下面的结构填写：

```markdown
# Phase 0 兼容性记录

- 日期：2026-10-22
- rustc：<rustc --version 的原文>
- cargo：<cargo --version 的原文>
- `cargo test --workspace` 退出码：<实际数字>

## 已执行

<粘贴 cargo test --workspace 的结果摘要，包括通过的测试数量>

## 未验证

- Android JNI
- HarmonyOS SDK、DevEco Studio、NAPI、真机打包
- Tauri 2、React、SQLx
- LLM 网络调用
- 同步 API
```

把 `docs/README_AI_CODING.md` 的「当前阶段」改为：

```markdown
## 当前阶段

Phase 0 的实施计划见 `superpowers/plans/2026-10-22-phase0-rust-workspace.md`。兼容性结果以 `compatibility/phase0.md` 为准。不要直接生成 Windows、Android、HarmonyOS 或同步服务。
```

- [ ] **Step 4: 对照文档检查边界**

确认仓库中不存在 `apps/`、`services/`、`pyproject.toml`。确认 `crates/workpilot-domain/Cargo.toml` 没有 `serde`、`sqlx`、`reqwest` 或 `tauri`。

- [ ] **Step 5: 保留变更**

不执行 git commit。向用户列出新增文件、实际命令、退出码和未验证项。
