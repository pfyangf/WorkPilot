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
