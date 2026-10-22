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
    let mut report =
        DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
    report.regenerate("根据日志重写", at(10)).unwrap();
    assert_eq!(report.content(), "根据日志重写");
    assert_eq!(report.content_origin(), ContentOrigin::Generated);
    assert_eq!(report.version_no(), 1);
    assert_eq!(report.version(), 2);
}

#[test]
fn regenerate_does_not_overwrite_user_edits() {
    let mut report =
        DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
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
    let mut report =
        DailyReport::create_generated(today(), "2026-10-22 日报", "初稿", at(9)).unwrap();
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
