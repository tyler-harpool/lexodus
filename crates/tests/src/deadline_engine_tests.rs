//! FRCP 6(a) deadline computation tests
//! Ported from spin-lexodus

use chrono::NaiveDate;
use server::compliance::deadline_engine::*;
use shared_types::compliance::{DeadlineComputeRequest, ServiceMethod};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn make_request(
    trigger: NaiveDate,
    period: i32,
    service: ServiceMethod,
) -> DeadlineComputeRequest {
    DeadlineComputeRequest {
        trigger_date: trigger,
        period_days: period,
        service_method: service,
        jurisdiction: "TEST".to_string(),
        description: "Test deadline".to_string(),
        rule_citation: "FRCP 12(a)".to_string(),
    }
}

#[test]
fn holiday_list_has_eleven_entries() {
    assert_eq!(get_federal_holidays(2025).len(), 11);
}

#[test]
fn mlk_day_2025_is_jan_20() {
    let holidays = get_federal_holidays(2025);
    let mlk = holidays.iter().find(|h| h.name.contains("King")).unwrap();
    assert_eq!(mlk.date, date(2025, 1, 20));
}

#[test]
fn memorial_day_2025_is_may_26() {
    let holidays = get_federal_holidays(2025);
    let mem = holidays.iter().find(|h| h.name.contains("Memorial")).unwrap();
    assert_eq!(mem.date, date(2025, 5, 26));
}

#[test]
fn july_4_2026_saturday_observed_friday() {
    // July 4, 2026 is a Saturday; observed on Friday July 3
    assert!(is_federal_holiday(date(2026, 7, 3)));
    assert!(!is_federal_holiday(date(2026, 7, 4)));
}

#[test]
fn saturday_is_weekend() {
    assert!(is_weekend(date(2025, 10, 4)));
}

#[test]
fn monday_is_not_weekend() {
    assert!(!is_weekend(date(2025, 10, 6)));
}

#[test]
fn next_business_day_on_weekday_unchanged() {
    assert_eq!(next_business_day(date(2025, 10, 8)), date(2025, 10, 8));
}

#[test]
fn next_business_day_on_saturday_goes_to_monday() {
    assert_eq!(next_business_day(date(2025, 10, 4)), date(2025, 10, 6));
}

#[test]
fn next_business_day_on_holiday_skips() {
    // Christmas 2025 is Thursday
    assert_eq!(next_business_day(date(2025, 12, 25)), date(2025, 12, 26));
}

#[test]
fn five_day_period_lands_on_weekend_then_holiday() {
    // 5-day from Mon Oct 6, 2025
    // Start: Oct 7, +4 days = Oct 11 (Sat) -> Mon Oct 13 (Columbus Day) -> Tue Oct 14
    let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 10, 14));
    assert!(result.is_short_period);
}

#[test]
fn five_day_with_mail_adds_three() {
    // 5 + 3 = 8 days, from Oct 6
    // Start: Oct 7, +7 = Oct 14 (Tue, Columbus Day observed Mon Oct 13)
    let req = make_request(date(2025, 10, 6), 5, ServiceMethod::Mail);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 10, 14));
    assert!(result.is_short_period);
}

#[test]
fn thirty_day_period() {
    // 30-day from Oct 7: start Oct 8, +29 = Nov 6 (Thu)
    let req = make_request(date(2025, 10, 7), 30, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 11, 6));
    assert!(!result.is_short_period);
}

#[test]
fn landing_on_christmas_extends() {
    // 31-day from Nov 25: start Nov 26, +30 = Dec 25 (Thu Christmas) -> Dec 26
    let req = make_request(date(2025, 11, 25), 31, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    assert_eq!(result.due_date, date(2025, 12, 26));
}

#[test]
fn zero_day_period() {
    let req = make_request(date(2025, 10, 6), 0, ServiceMethod::Electronic);
    let result = compute_deadline(&req).unwrap();
    // Zero period: start = Oct 7, count_calendar_days(Oct 7, 0) = Oct 7
    assert_eq!(result.due_date, date(2025, 10, 7));
}

#[test]
fn negative_period_is_error() {
    let req = make_request(date(2025, 10, 6), -1, ServiceMethod::Electronic);
    assert!(compute_deadline(&req).is_err());
}
