use crate::connector_error::ConnectorError;
use chrono::{DateTime, FixedOffset, Local, Utc};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VDTOffset {
    Utc(DateTime<Utc>),
    Local(DateTime<Local>),
    FixedOffset(DateTime<FixedOffset>),
}

#[derive(Builder, Clone, Debug, PartialEq)]
pub struct ValidDateTime {
    #[builder(setter(into))]
    pub start_date_time: VDTOffset,

    #[builder(setter(strip_option), default)]
    pub period_date: Option<PeriodDate>,

    #[builder(setter(strip_option), default)]
    pub end_date_time: Option<VDTOffset>,

    #[builder(setter(strip_option), default)]
    pub time_step: Option<PeriodTime>,

    #[builder(setter(strip_option), default)]
    pub time_list: Option<Vec<VDTOffset>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PeriodDate {
    Years(i32),
    Months(i32),
    Days(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PeriodTime {
    Hours(i32),
    Minutes(i32),
    Seconds(i32),
}

impl ValidDateTime {
    pub fn format(&self) -> Result<String, ConnectorError> {
        let start = self.start_date_time.to_string();
        let mut suffix = "".to_string();
        let mut both = false;
        if self.period_date.is_some() {
            suffix = self.period_date.unwrap().to_string();
        }
        if self.time_step.is_some() {
            if !suffix.is_empty() {
                suffix += ":";
                both = true;
            }
            suffix += &*self.time_step.unwrap().to_string();
        }
        match self.end_date_time {
            None => Ok(start + &*suffix),
            Some(_) => {
                if both {
                    return Err(ConnectorError::LibraryError(
                        "Cannot use period date and time step simultaneously.".to_string(),
                    ));
                }
                let mut end = self.end_date_time.unwrap().to_string();
                if !suffix.is_empty() {
                    end = end + ":" + &*suffix;
                }
                Ok(format!("{}--{}", start, end))
            }
        }
    }
}

impl Display for VDTOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VDTOffset::Utc(dt) => write!(f, "{}", dt.to_rfc3339()),
            VDTOffset::Local(dt) => write!(f, "{}", dt.to_rfc3339()),
            VDTOffset::FixedOffset(dt) => write!(f, "{}", dt.to_rfc3339()),
        }
    }
}

impl Display for PeriodDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodDate::Years(n) => write!(f, "P{}Y", n),
            PeriodDate::Months(n) => write!(f, "P{}M", n),
            PeriodDate::Days(n) => write!(f, "P{}D", n),
        }
    }
}

impl Display for PeriodTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodTime::Hours(n) => write!(f, "PT{}H", n),
            PeriodTime::Minutes(n) => write!(f, "PT{}M", n),
            PeriodTime::Seconds(n) => write!(f, "PT{}S", n),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::valid_date_time::{
        PeriodDate, PeriodTime, VDTOffset, ValidDateTime, ValidDateTimeBuilder,
    };
    use chrono::{Duration, Local, Utc};

    #[tokio::test]
    async fn create_with_default() {
        println!("\n##### create_with_default (UTC):");

        // Use UTC.
        let start_date_time = VDTOffset::Utc(Utc::now());

        let utc_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(start_date_time)
            .build()
            .unwrap();

        println!(
            ">>>>>>>>>> utc_vdt.start_date_time: {:?}",
            utc_vdt.start_date_time
        );
        println!(
            ">>>>>>>>>> utc_vdt.start_date_time: {}",
            utc_vdt.start_date_time
        );
        println!(">>>>>>>>>> utc_vdt.period_date: {:?}", utc_vdt.period_date);
        println!(
            ">>>>>>>>>> utc_vdt.end_date_time: {:?}",
            utc_vdt.end_date_time
        );
        println!(">>>>>>>>>> utc_vdt.time_step: {:?}", utc_vdt.time_step);

        assert_eq!(
            utc_vdt,
            ValidDateTime {
                start_date_time,
                period_date: None,
                end_date_time: None,
                time_step: None,
                time_list: None
            }
        );
    }

    #[tokio::test]
    async fn create_with_optional_params() {
        println!("\n##### create_with_optional_params (local):");

        // Use local time zone.
        let start_date_time = Local::now();
        let period_date = PeriodDate::Days(1);
        let end_date_time = start_date_time.clone() + Duration::days(1);
        let time_step = PeriodTime::Hours(1);
        let start_vdt_offset = VDTOffset::Local(start_date_time);
        let end_vdt_offset = VDTOffset::Local(end_date_time);
        let time_list = vec![start_vdt_offset, end_vdt_offset];

        let local_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(start_vdt_offset)
            .period_date(period_date)
            .end_date_time(end_vdt_offset)
            .time_step(time_step)
            .time_list(time_list)
            .build()
            .unwrap();

        println!(
            ">>>>>>>>>> local_vdt.start_date_time: {:?}",
            local_vdt.start_date_time
        );
        println!(
            ">>>>>>>>>> local_vdt.start_date_time: {}",
            local_vdt.start_date_time
        );
        println!(
            ">>>>>>>>>> local_vdt.period_date: {}",
            local_vdt.period_date.unwrap()
        );
        println!(
            ">>>>>>>>>> local_vdt.end_date_time: {:?}",
            local_vdt.end_date_time.unwrap()
        );
        println!(
            ">>>>>>>>>> local_vdt.end_date_time: {}",
            local_vdt.end_date_time.unwrap()
        );
        println!(
            ">>>>>>>>>> local_vdt.time_step: {}",
            local_vdt.time_step.unwrap()
        );

        let tl = local_vdt.time_list.unwrap();
        println!(">>>>>>>>>> local_vdt.time_list: {:?}", tl);

        assert_eq!(local_vdt.start_date_time, start_vdt_offset);
        assert_eq!(local_vdt.end_date_time.unwrap(), end_vdt_offset);

        assert_eq!(local_vdt.period_date.unwrap(), PeriodDate::Days(1));
        assert_eq!(local_vdt.time_step.unwrap(), PeriodTime::Hours(1));

        assert_eq!(local_vdt.period_date.unwrap().to_string(), "P1D");
        assert_eq!(local_vdt.time_step.unwrap().to_string(), "PT1H");

        assert_eq!(tl[0], start_vdt_offset);
        assert_eq!(tl[1], end_vdt_offset);
    }
}
