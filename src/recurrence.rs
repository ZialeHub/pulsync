use std::time::Duration;

use chrono::NaiveDateTime;

/// Recurrence of a task
///
/// The recurrence of a task is the time interval between each execution of the task.
/// The unit of the recurrence is in seconds.
/// And the count variable (optional) is the number of times the task should be executed
///
/// If the count is set to None, the task will be executed indefinitely.
///
/// The parameter run_after is used to determine if the task should be executed after the first interval.
///
/// # Example
/// ```rust,ignore
/// // Execute the task every second for 3 times
/// let recurrence = every(1.seconds()).count(3);
///
/// // Execute the task every 1 minute and 2 seconds
/// let recurrence = every(1.minutes()).and(2.seconds());
/// ```
#[derive(Debug, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Recurrence {
    pub(crate) unit: RecurrenceUnit,
    pub(crate) run_after: bool,
    pub(crate) count: Option<u64>,
    pub(crate) limit: Option<RecurrenceUnit>,
    pub(crate) limit_datetime: Option<NaiveDateTime>,
}
impl Recurrence {
    /// Increase the recurrence by a number of seconds
    ///
    /// # Example
    /// ```rust,ignore
    /// // Execute the task every 1 minute and 2 seconds
    /// let recurrence = every(1.minutes()).and(2.seconds());
    /// ```
    pub fn and(mut self, unit: RecurrenceUnit) -> Self {
        *self.unit += *unit;
        self
    }

    /// Set the number of times the task should be executed
    ///
    /// By default, the count is set to None (infinite)
    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }

    /// Set if the task should be executed at run,
    /// or if it should be delayed by 1 recurrence timer.
    ///
    /// By default, the value is set to false
    pub fn run_after(mut self, value: bool) -> Self {
        self.run_after = value;
        self
    }

    /// Set the time interval when the recurrence should stop
    ///
    /// By default, the value is set to None
    ///
    /// # Example
    /// ```rust,ignore
    /// // Execute the task every second for 5 seconds
    /// let recurrence = every(1.seconds()).until(5.seconds());
    ///
    /// // Execute the task every 1 minute for 10 minutes
    /// let recurrence = every(1.minutes()).until(10.minutes());
    /// ```
    pub fn until(mut self, value: RecurrenceUnit) -> Self {
        self.limit = Some(value);
        self
    }

    /// Set the datetime when the recurrence should stop
    ///
    /// By default, the value is set to None
    ///
    /// # Example
    /// ```rust,ignore
    /// // Execute the task every second until 2021-01-01 00:00:00
    /// let recurrence = every(1.seconds()).until_datetime(NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")?);
    ///
    /// // Execute the task every 1 minute until 2021-01-01 00:00:00
    /// let recurrence = every(1.minutes()).until_datetime(NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")?);
    /// ```
    pub fn until_datetime(mut self, value: NaiveDateTime) -> Self {
        self.limit_datetime = Some(value);
        self
    }
}

/// Create a new recurrence with a number of seconds
///
/// By default, the count is set to None (infinite)
///
/// # Example
/// ```rust,ignore
/// // Execute the task every second
/// let recurrence = every(1.seconds());
///
/// // Execute the task every 1 minute
/// let recurrence = every(1.minutes());
/// ```
pub fn every(unit: RecurrenceUnit) -> Recurrence {
    Recurrence {
        unit,
        run_after: false,
        count: None,
        limit: None,
        limit_datetime: None,
    }
}

/// RecurrenceUnit is a wrapper around u64 to represent the time interval between each execution of a task.
///
/// It implements the Into trait to convert the RecurrenceUnit into a Duration used to wait for the next execution of the task.
#[derive(Debug, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecurrenceUnit(u64);

impl std::ops::Deref for RecurrenceUnit {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RecurrenceUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<RecurrenceUnit> for Duration {
    fn from(value: RecurrenceUnit) -> Self {
        Duration::from_secs(value.0)
    }
}

/// RecurrenceCast is a trait to cast a number into a RecurrenceUnit
///
/// It provides methods to convert a number into a RecurrenceUnit with a specific time unit.
pub trait RecurrenceCast
where
    Self: Into<u64>,
{
    fn seconds(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into())
    }

    fn minutes(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60)
    }

    fn hours(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60)
    }

    fn days(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24)
    }

    fn weeks(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 7)
    }

    fn months(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 30)
    }

    fn years(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 365)
    }
}

impl RecurrenceCast for u64 {}
