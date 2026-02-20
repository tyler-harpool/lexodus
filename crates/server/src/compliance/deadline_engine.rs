//! FRCP Rule 6(a) deadline computation engine
//!
//! Ported from spin-lexodus. Computes deadlines per FRCP 6(a)(1)
//! (effective Dec 1, 2009). All periods use calendar-day counting.
//! Landing day on weekend/holiday extends to next business day.
//! Service method adjustments per FRCP 6(d).

use chrono::{Datelike, NaiveDate, Weekday};
use shared_types::compliance::{
    DeadlineComputeRequest, DeadlineResult, FederalHoliday,
};

const SHORT_PERIOD_THRESHOLD: i32 = 14;

/// Compute a deadline per FRCP 6(a).
pub fn compute_deadline(request: &DeadlineComputeRequest) -> Result<DeadlineResult, String> {
    if request.period_days < 0 {
        return Err("Period days cannot be negative".to_string());
    }

    let service_additional = request.service_method.additional_days();
    let total_period = request.period_days + service_additional;
    let is_short = total_period <= SHORT_PERIOD_THRESHOLD;

    // Step 1: Exclude trigger date -- start from next day
    let start_date = request
        .trigger_date
        .succ_opt()
        .ok_or("Trigger date overflow")?;

    // Step 2: Count ALL calendar days (FRCP 6(a)(1))
    let raw_due_date = count_calendar_days(start_date, total_period)?;

    // Step 3: If due date falls on weekend or holiday, extend
    let due_date = next_business_day(raw_due_date);

    let mut notes = Vec::new();
    notes.push(format!(
        "Trigger date: {}; counting begins {}",
        request.trigger_date, start_date
    ));

    if service_additional > 0 {
        notes.push(format!(
            "Service method ({:?}): +{} days added to base period of {} days",
            request.service_method, service_additional, request.period_days
        ));
    }

    notes.push(format!(
        "Total period: {} calendar days{}",
        total_period,
        if is_short {
            " (short period per FRCP 6(a)(2))"
        } else {
            ""
        }
    ));

    if due_date != raw_due_date {
        notes.push(format!(
            "Landing day {} falls on weekend/holiday; extended to next business day {}",
            raw_due_date, due_date
        ));
    }

    notes.push(format!("Due date: {}", due_date));

    Ok(DeadlineResult {
        due_date,
        description: request.description.clone(),
        rule_citation: request.rule_citation.clone(),
        computation_notes: notes.join("; "),
        is_short_period: is_short,
    })
}

/// Check if a date is a federal holiday.
pub fn is_federal_holiday(date: NaiveDate) -> bool {
    get_federal_holidays(date.year())
        .iter()
        .any(|h| h.date == date)
}

/// Check if a date is a weekend.
pub fn is_weekend(date: NaiveDate) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Find the next business day (skipping weekends and federal holidays).
pub fn next_business_day(date: NaiveDate) -> NaiveDate {
    let mut current = date;
    while is_weekend(current) || is_federal_holiday(current) {
        current = current.succ_opt().unwrap_or(current);
    }
    current
}

/// Count calendar days from a start date per FRCP 6(a)(1).
fn count_calendar_days(start: NaiveDate, days: i32) -> Result<NaiveDate, String> {
    if days <= 0 {
        return Ok(start);
    }
    start
        .checked_add_signed(chrono::Duration::days((days - 1) as i64))
        .ok_or_else(|| "Date overflow during calendar day count".to_string())
}

/// Compute the nth occurrence of a given weekday in a month.
fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
    let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let first_weekday = first_of_month.weekday();
    let days_ahead = (weekday.num_days_from_monday() as i32
        - first_weekday.num_days_from_monday() as i32
        + 7) % 7;
    let day = 1 + days_ahead as u32 + (n - 1) * 7;
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Compute the last occurrence of a given weekday in a month.
fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    let last_of_month = last_day.pred_opt().unwrap();
    let last_weekday = last_of_month.weekday();
    let days_back = (last_weekday.num_days_from_monday() as i32
        - weekday.num_days_from_monday() as i32
        + 7) % 7;
    NaiveDate::from_ymd_opt(year, month, last_of_month.day() - days_back as u32).unwrap()
}

/// Apply federal holiday observation rule (Sat->Friday, Sun->Monday).
fn observed_date(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date.pred_opt().unwrap(),
        Weekday::Sun => date.succ_opt().unwrap(),
        _ => date,
    }
}

fn add_observed_holiday(
    holidays: &mut Vec<FederalHoliday>,
    year: i32,
    month: u32,
    day: u32,
    name: &str,
) {
    let actual = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    let obs = observed_date(actual);
    holidays.push(FederalHoliday {
        date: obs,
        name: name.to_string(),
    });
}

/// Get all federal holidays for a given year (11 holidays).
pub fn get_federal_holidays(year: i32) -> Vec<FederalHoliday> {
    let mut holidays = Vec::new();

    // New Year's Day -- January 1
    add_observed_holiday(&mut holidays, year, 1, 1, "New Year's Day");

    // Martin Luther King Jr. Day -- third Monday of January
    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 1, Weekday::Mon, 3),
        name: "Martin Luther King Jr. Day".to_string(),
    });

    // Presidents' Day -- third Monday of February
    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 2, Weekday::Mon, 3),
        name: "Presidents' Day".to_string(),
    });

    // Memorial Day -- last Monday of May
    holidays.push(FederalHoliday {
        date: last_weekday_of_month(year, 5, Weekday::Mon),
        name: "Memorial Day".to_string(),
    });

    // Juneteenth -- June 19
    add_observed_holiday(&mut holidays, year, 6, 19, "Juneteenth");

    // Independence Day -- July 4
    add_observed_holiday(&mut holidays, year, 7, 4, "Independence Day");

    // Labor Day -- first Monday of September
    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 9, Weekday::Mon, 1),
        name: "Labor Day".to_string(),
    });

    // Columbus Day -- second Monday of October
    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 10, Weekday::Mon, 2),
        name: "Columbus Day".to_string(),
    });

    // Veterans Day -- November 11
    add_observed_holiday(&mut holidays, year, 11, 11, "Veterans Day");

    // Thanksgiving Day -- fourth Thursday of November
    holidays.push(FederalHoliday {
        date: nth_weekday_of_month(year, 11, Weekday::Thu, 4),
        name: "Thanksgiving Day".to_string(),
    });

    // Christmas Day -- December 25
    add_observed_holiday(&mut holidays, year, 12, 25, "Christmas Day");

    holidays.sort_by_key(|h| h.date);
    holidays
}
