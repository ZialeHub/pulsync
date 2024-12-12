use std::time::Duration;

pub struct Recurrence {
    pub unit: RecurrenceUnit,
    pub count: Option<u64>,
}
impl Recurrence {
    pub fn and(mut self, unit: RecurrenceUnit) -> Self {
        *self.unit = *self.unit + *unit;
        self
    }

    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }
}

pub fn every(unit: RecurrenceUnit) -> Recurrence {
    Recurrence { unit, count: None }
}

/// Recurrence in seconds
#[derive(Clone, Copy)]
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
impl Into<Duration> for RecurrenceUnit {
    fn into(self) -> Duration {
        Duration::from_secs(self.0)
    }
}

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

// TODO Find a way to implement this for all unsigned integers
// For now, it creates a conflict because the compiler can't decide which type to use
//impl RecurrenceCast for u8 {}
//impl RecurrenceCast for u16 {}
//impl RecurrenceCast for u32 {}
impl RecurrenceCast for u64 {}
