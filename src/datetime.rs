use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};

/// assumes naivedatetime is in UTC timezone
pub fn naivedatetime_to_utc(naive_datetime: NaiveDateTime) -> DateTime<Utc> {
	let utc_dt: DateTime<Utc> = Utc.from_utc_datetime(&naive_datetime);
	utc_dt
}

/// assumes naivedatetime is in Local timezone
pub fn naivedatetime_to_local(naive_datetime: NaiveDateTime) -> DateTime<Local> {
	Local
		.from_local_datetime(&naive_datetime)
		.earliest() // Choose the earliest time in case of DST ambiguity (a "fold")
		.unwrap_or_else(|| {
			// This case handles a non-existent time during a DST spring-forward gap.
			// If the date is one that falls in the gap, we return the NaiveDateTime
			// as the next hour
			let local_dt = Local.with_ymd_and_hms(naive_datetime.year(), naive_datetime.month(), naive_datetime.day(), naive_datetime.hour()+1, 0, 0).unwrap();
			local_dt
		})
}

/// assumes naivedate is in UTC timezone at midnight
pub fn naivedate_to_utc(naive_date: NaiveDate) -> DateTime<Utc> {
	let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("Midnight is always valid");
	let naive_datetime = naive_date.and_time(midnight);
	let utc_dt: DateTime<Utc> = naivedatetime_to_utc(naive_datetime);
	utc_dt
}

/// assumes naivedate is in Local timezone at midnight
pub fn naivedate_to_local(naive_date: NaiveDate) -> DateTime<Local> {
	let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("Midnight is always valid");
	let naive_datetime = naive_date.and_time(midnight);
	let local_dt: DateTime<Local> = naivedatetime_to_local(naive_datetime);
	local_dt
}



#[cfg(test)]
mod tests {
    //https://docs.rs/chrono/latest/chrono/format/strftime/index.html
	
	use super::*;

    #[test]
    fn test_naivedatetime_to_utc() {
		let naive_datetime = NaiveDateTime::parse_from_str(
			"2025-11-15 15:30:24", 
			"%Y-%m-%d %H:%M:%S"
    	).expect("Failed to parse NaiveDateTime");
		let result = naivedatetime_to_utc(naive_datetime);
		let expected: DateTime<Utc> = DateTime::parse_from_str(
			"2025-11-15 15:30:24 +0000",
			"%Y-%m-%d %H:%M:%S %z"
		).expect("Failed to parse DateTime<Utc>").into();

		assert_eq!(result, expected);
    }

    #[test]
    fn test_naivedatetime_to_local() {
		let naive_datetime = NaiveDateTime::parse_from_str(
			"2025-11-15 15:30:24", 
			"%Y-%m-%d %H:%M:%S"
    	).expect("Failed to parse NaiveDateTime");
		let result = naivedatetime_to_local(naive_datetime);
		let expected: DateTime<Local> = Local.with_ymd_and_hms(2025, 11, 15, 15, 30, 24).unwrap();

		assert_eq!(result, expected);
    }

    #[test]
    fn test_naivedatetime_to_local_spring_forward() {
		//in NZT, 2025-09-28 02:30 doesn't exist (clocks jump from 01:59 -> 03:00). Should return next real time, 03:00
		// last Sunday in September, 2025-09-28 02:00
		let naive_datetime = NaiveDateTime::parse_from_str(
			"2025-09-28 02:30:00", 
			"%Y-%m-%d %H:%M:%S"
    	).expect("Failed to parse NaiveDateTime");
		let result = naivedatetime_to_local(naive_datetime);
		let expected: DateTime<Local> = Local.with_ymd_and_hms(2025, 09, 28, 03, 0, 0).unwrap();

		assert_eq!(result, expected);
    }

    #[test]
    fn test_naivedate_to_utc() {
		let naive_date = NaiveDate::parse_from_str(
			"2025-11-15",
			"%Y-%m-%d"
    	).expect("Failed to parse NaiveDate");
		let result = naivedate_to_utc(naive_date);
		let expected: DateTime<Utc> = DateTime::parse_from_str(
			"2025-11-15 00:00:00 +0000",
			"%Y-%m-%d %H:%M:%S %z"
		).expect("Failed to parse DateTime<Utc>").into();

		assert_eq!(result, expected);
    }

    #[test]
    fn test_naivedate_to_local() {
		let naive_date = NaiveDate::parse_from_str(
			"2025-11-15", 
			"%Y-%m-%d"
    	).expect("Failed to parse NaiveDateTime");
		let result = naivedate_to_local(naive_date);
		let expected: DateTime<Local> = Local.with_ymd_and_hms(2025, 11, 15, 0, 0, 0).unwrap();

		assert_eq!(result, expected);
    }

}
