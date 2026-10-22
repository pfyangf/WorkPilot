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
    // 创建为 1，状态变更为 2，两次追加日志后为 4。
    assert_eq!(task.version(), 4);
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
