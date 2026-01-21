use core::pin::Pin;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, input_timestamp)]
        #[qproperty(QString, input_datetime)]
        #[qproperty(QString, input_timezone)]
        #[qproperty(QString, output_timestamp)]
        #[qproperty(QString, output_iso8601)]
        #[qproperty(QString, output_rfc2822)]
        #[qproperty(QString, output_relative)]
        #[qproperty(QString, output_local)]
        #[qproperty(QString, target_timezone)]
        #[qproperty(QString, converted_time)]
        #[qproperty(i64, add_seconds)]
        #[qproperty(i64, add_minutes)]
        #[qproperty(i64, add_hours)]
        #[qproperty(i64, add_days)]
        #[qproperty(QString, arithmetic_result)]
        #[qproperty(QString, current_time)]
        #[qproperty(QString, error_message)]
        type TimeModel = super::TimeModelRust;

        #[qinvokable]
        fn parse_timestamp(self: Pin<&mut TimeModel>);

        #[qinvokable]
        fn parse_datetime(self: Pin<&mut TimeModel>);

        #[qinvokable]
        fn get_current_time(self: Pin<&mut TimeModel>);

        #[qinvokable]
        fn convert_timezone(self: Pin<&mut TimeModel>);

        #[qinvokable]
        fn apply_arithmetic(self: Pin<&mut TimeModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut TimeModel>);
    }
}

pub struct TimeModelRust {
    input_timestamp: QString,
    input_datetime: QString,
    input_timezone: QString,
    output_timestamp: QString,
    output_iso8601: QString,
    output_rfc2822: QString,
    output_relative: QString,
    output_local: QString,
    target_timezone: QString,
    converted_time: QString,
    add_seconds: i64,
    add_minutes: i64,
    add_hours: i64,
    add_days: i64,
    arithmetic_result: QString,
    current_time: QString,
    error_message: QString,
}

impl Default for TimeModelRust {
    fn default() -> Self {
        Self {
            input_timestamp: QString::from(""),
            input_datetime: QString::from(""),
            input_timezone: QString::from("UTC"),
            output_timestamp: QString::from(""),
            output_iso8601: QString::from(""),
            output_rfc2822: QString::from(""),
            output_relative: QString::from(""),
            output_local: QString::from(""),
            target_timezone: QString::from("America/New_York"),
            converted_time: QString::from(""),
            add_seconds: 0,
            add_minutes: 0,
            add_hours: 0,
            add_days: 0,
            arithmetic_result: QString::from(""),
            current_time: QString::from(""),
            error_message: QString::from(""),
        }
    }
}

impl qobject::TimeModel {
    pub fn parse_timestamp(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input_timestamp().to_string();

        if input.is_empty() {
            self.as_mut().clear_outputs();
            return;
        }

        // Try to parse as Unix timestamp (seconds or milliseconds)
        let timestamp: i64 = match input.trim().parse() {
            Ok(ts) => ts,
            Err(_) => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid timestamp format"));
                return;
            }
        };

        // Detect if milliseconds (13+ digits) or seconds
        let datetime = if timestamp > 9999999999 {
            // Milliseconds
            match DateTime::from_timestamp_millis(timestamp) {
                Some(dt) => dt,
                None => {
                    self.as_mut()
                        .set_error_message(QString::from("Invalid timestamp value"));
                    return;
                }
            }
        } else {
            // Seconds
            match DateTime::from_timestamp(timestamp, 0) {
                Some(dt) => dt,
                None => {
                    self.as_mut()
                        .set_error_message(QString::from("Invalid timestamp value"));
                    return;
                }
            }
        };

        self.as_mut().set_outputs_from_datetime(datetime);
    }

    pub fn parse_datetime(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input_datetime().to_string();
        let tz_str = self.as_ref().input_timezone().to_string();

        if input.is_empty() {
            self.as_mut().clear_outputs();
            return;
        }

        // Try various datetime formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%d",
            "%d/%m/%Y %H:%M:%S",
            "%m/%d/%Y %H:%M:%S",
        ];

        let mut parsed: Option<NaiveDateTime> = None;
        for fmt in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(input.trim(), fmt) {
                parsed = Some(dt);
                break;
            }
        }

        // Also try date-only formats
        if parsed.is_none() {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d") {
                parsed = Some(date.and_hms_opt(0, 0, 0).unwrap());
            }
        }

        let naive_dt = match parsed {
            Some(dt) => dt,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Could not parse datetime. Try: YYYY-MM-DD HH:MM:SS"));
                return;
            }
        };

        // Apply timezone
        let datetime: DateTime<Utc> = if tz_str == "UTC" {
            naive_dt.and_utc()
        } else if tz_str == "Local" {
            match Local.from_local_datetime(&naive_dt).single() {
                Some(dt) => dt.with_timezone(&Utc),
                None => {
                    self.as_mut()
                        .set_error_message(QString::from("Ambiguous local time"));
                    return;
                }
            }
        } else {
            match tz_str.parse::<Tz>() {
                Ok(tz) => match tz.from_local_datetime(&naive_dt).single() {
                    Some(dt) => dt.with_timezone(&Utc),
                    None => {
                        self.as_mut()
                            .set_error_message(QString::from("Ambiguous time in timezone"));
                        return;
                    }
                },
                Err(_) => {
                    self.as_mut()
                        .set_error_message(QString::from("Invalid timezone"));
                    return;
                }
            }
        };

        self.as_mut().set_outputs_from_datetime(datetime);
    }

    pub fn get_current_time(mut self: Pin<&mut Self>) {
        let now = Utc::now();
        let local = Local::now();

        let current = format!(
            "UTC: {}\nLocal: {}\nTimestamp: {}",
            now.format("%Y-%m-%d %H:%M:%S UTC"),
            local.format("%Y-%m-%d %H:%M:%S %Z"),
            now.timestamp()
        );

        self.as_mut().set_current_time(QString::from(&current));
        self.as_mut().set_outputs_from_datetime(now);
    }

    pub fn convert_timezone(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let timestamp_str = self.as_ref().output_timestamp().to_string();
        let target_tz = self.as_ref().target_timezone().to_string();

        if timestamp_str.is_empty() {
            self.as_mut().set_converted_time(QString::from(""));
            return;
        }

        let timestamp: i64 = match timestamp_str.parse() {
            Ok(ts) => ts,
            Err(_) => {
                self.as_mut()
                    .set_error_message(QString::from("No timestamp to convert"));
                return;
            }
        };

        let datetime = match DateTime::from_timestamp(timestamp, 0) {
            Some(dt) => dt,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid timestamp"));
                return;
            }
        };

        let converted = if target_tz == "Local" {
            datetime
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S %Z")
                .to_string()
        } else {
            match target_tz.parse::<Tz>() {
                Ok(tz) => datetime
                    .with_timezone(&tz)
                    .format("%Y-%m-%d %H:%M:%S %Z")
                    .to_string(),
                Err(_) => {
                    self.as_mut()
                        .set_error_message(QString::from("Invalid target timezone"));
                    return;
                }
            }
        };

        self.as_mut().set_converted_time(QString::from(&converted));
    }

    pub fn apply_arithmetic(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let timestamp_str = self.as_ref().output_timestamp().to_string();

        if timestamp_str.is_empty() {
            self.as_mut().set_arithmetic_result(QString::from(""));
            return;
        }

        let timestamp: i64 = match timestamp_str.parse() {
            Ok(ts) => ts,
            Err(_) => {
                self.as_mut()
                    .set_error_message(QString::from("No timestamp for arithmetic"));
                return;
            }
        };

        let datetime = match DateTime::from_timestamp(timestamp, 0) {
            Some(dt) => dt,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid timestamp"));
                return;
            }
        };

        let seconds = self.as_ref().add_seconds();
        let minutes = self.as_ref().add_minutes();
        let hours = self.as_ref().add_hours();
        let days = self.as_ref().add_days();

        let total_seconds = seconds + (minutes * 60) + (hours * 3600) + (days * 86400);
        let duration = chrono::Duration::seconds(total_seconds);

        let new_datetime = datetime + duration;
        let result = format!(
            "New timestamp: {}\nISO 8601: {}\nLocal: {}",
            new_datetime.timestamp(),
            new_datetime.to_rfc3339(),
            new_datetime.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S %Z")
        );

        self.as_mut().set_arithmetic_result(QString::from(&result));
    }

    fn set_outputs_from_datetime(mut self: Pin<&mut Self>, datetime: DateTime<Utc>) {
        let timestamp = datetime.timestamp();
        let iso8601 = datetime.to_rfc3339();
        let rfc2822 = datetime.to_rfc2822();
        let local = datetime
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string();

        // Calculate relative time
        let now = Utc::now();
        let diff = now.signed_duration_since(datetime);
        let relative = Self::format_relative_time(diff);

        self.as_mut()
            .set_output_timestamp(QString::from(&timestamp.to_string()));
        self.as_mut().set_output_iso8601(QString::from(&iso8601));
        self.as_mut().set_output_rfc2822(QString::from(&rfc2822));
        self.as_mut().set_output_relative(QString::from(&relative));
        self.as_mut().set_output_local(QString::from(&local));
    }

    fn format_relative_time(duration: chrono::Duration) -> String {
        let seconds = duration.num_seconds().abs();
        let is_past = duration.num_seconds() > 0;
        let suffix = if is_past { "ago" } else { "from now" };

        if seconds < 60 {
            format!("{} seconds {}", seconds, suffix)
        } else if seconds < 3600 {
            let minutes = seconds / 60;
            format!("{} minute{} {}", minutes, if minutes == 1 { "" } else { "s" }, suffix)
        } else if seconds < 86400 {
            let hours = seconds / 3600;
            format!("{} hour{} {}", hours, if hours == 1 { "" } else { "s" }, suffix)
        } else if seconds < 2592000 {
            let days = seconds / 86400;
            format!("{} day{} {}", days, if days == 1 { "" } else { "s" }, suffix)
        } else if seconds < 31536000 {
            let months = seconds / 2592000;
            format!("{} month{} {}", months, if months == 1 { "" } else { "s" }, suffix)
        } else {
            let years = seconds / 31536000;
            format!("{} year{} {}", years, if years == 1 { "" } else { "s" }, suffix)
        }
    }

    fn clear_outputs(mut self: Pin<&mut Self>) {
        self.as_mut().set_output_timestamp(QString::from(""));
        self.as_mut().set_output_iso8601(QString::from(""));
        self.as_mut().set_output_rfc2822(QString::from(""));
        self.as_mut().set_output_relative(QString::from(""));
        self.as_mut().set_output_local(QString::from(""));
        self.as_mut().set_converted_time(QString::from(""));
        self.as_mut().set_arithmetic_result(QString::from(""));
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input_timestamp(QString::from(""));
        self.as_mut().set_input_datetime(QString::from(""));
        self.as_mut().set_add_seconds(0);
        self.as_mut().set_add_minutes(0);
        self.as_mut().set_add_hours(0);
        self.as_mut().set_add_days(0);
        self.as_mut().clear_outputs();
        self.as_mut().set_current_time(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
