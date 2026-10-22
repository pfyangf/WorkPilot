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
