mod configuration;
mod connector_components;
mod entities;

pub use crate::connector_components::*;
pub use crate::entities::*;

use crate::configuration::api_client::APIClient;
use crate::connector_error::ConnectorError;
use crate::connector_response::ConnectorResponse;
use crate::locations::Locations;
use crate::optionals::Optionals;
use crate::parameters::Parameters;
use crate::valid_date_time::ValidDateTime;

#[macro_use]
extern crate derive_builder;

#[derive(Clone, Debug)]
pub struct MeteomaticsConnector {
    api_client: APIClient,
}

impl MeteomaticsConnector {
    pub fn new(username: String, password: String, timeout_seconds: u64) -> Self {
        Self {
            api_client: APIClient::new(username, password, timeout_seconds),
        }
    }

    pub async fn query_time_series(
        &self,
        vdt: ValidDateTime,
        parameters: Parameters<'_>,
        locations: Locations<'_>,
        optionals: Option<Optionals<'_>>,
    ) -> Result<ConnectorResponse, ConnectorError> {
        self.api_client
            .query_time_series(vdt, parameters, locations, optionals)
            .await
    }
}

#[cfg(test)]
mod tests {

    use crate::connector_response::ResponseBody;
    use crate::locations::{Coordinates, Locations};
    use crate::optionals::{Opt, OptSet, Optionals};
    use crate::parameters::{PSet, Parameters, P};
    use crate::valid_date_time::{
        PeriodDate, PeriodTime, VDTOffset, ValidDateTime, ValidDateTimeBuilder,
    };
    use crate::MeteomaticsConnector;
    use chrono::{Duration, Utc};
    use std::iter::FromIterator;

    #[tokio::test]
    async fn call_query_time_series_with_options() {
        println!("\n##### call_query_time_series_with_options:");

        // Create API connector
        let meteomatics_connector = MeteomaticsConnector::new(
            "python-community".to_string(),
            "Umivipawe179".to_string(),
            10,
        );

        // Create ValidDateTime
        let now = Utc::now();
        let yesterday = VDTOffset::Utc(now.clone() - Duration::days(1));
        let now = VDTOffset::Utc(now);
        let time_step = PeriodTime::Hours(1);
        let utc_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(yesterday)
            .end_date_time(now)
            .time_step(time_step)
            .build()
            .unwrap();

        // Create Parameters
        let parameters: Parameters = Parameters {
            p_values: PSet::from_iter([
                P {
                    k: "t_2m",
                    v: Some("C"),
                },
                P {
                    k: "precip_1h",
                    v: Some("mm"),
                },
            ]),
        };

        // Create Locations
        let locations: Locations = Locations {
            coordinates: Coordinates::from(["47.419708", "9.358478"]),
        };

        // Create Optionals
        let optionals: Optionals = Optionals {
            opt_values: OptSet::from_iter([
                Opt {
                    k: "source",
                    v: "mix",
                },
                Opt {
                    k: "calibrated",
                    v: "true",
                },
            ]),
        };

        // Call endpoint
        let result = meteomatics_connector
            .query_time_series(utc_vdt, parameters, locations, Option::from(optionals))
            .await;

        match result {
            Ok(ref response) => {
                let response_body = &response.response_body;
                println!("\n>>>>>>>>>> ResponseBody:\n{}", response_body);
                println!(
                    ">>>>>>>>>> ResponseHeaders:\n{}\n",
                    response_body.response_headers.to_vec().join(",")
                );
                println!(">>>>>>>>>> ResponseRecords:");
                for row in response_body.response_records.iter() {
                    let (index, values) = row;
                    let values_str: Vec<_> =
                        values.to_vec().iter().map(ToString::to_string).collect();
                    println!("{}", index.to_owned() + ": " + &values_str.join(","));
                }
                assert_eq!(response.http_status_code, "200");
                assert_eq!(response.http_status_message, "200 OK");
                assert_ne!(
                    response.response_body,
                    ResponseBody {
                        response_headers: vec![],
                        response_records: vec![]
                    }
                );
            }
            Err(ref connector_error) => {
                println!(">>>>>>>>>> ConnectorError: {:#?}", connector_error);
                assert!(result.is_err());
            }
        }

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn call_query_time_series_without_options() {
        println!("\n##### call_query_time_series_without_options:");

        // Create API connector
        let meteomatics_connector = MeteomaticsConnector::new(
            "python-community".to_string(),
            "Umivipawe179".to_string(),
            10,
        );

        // Create ValidDateTime
        let now = Utc::now();
        let yesterday = VDTOffset::Utc(now.clone() - Duration::days(1));
        let now = VDTOffset::Utc(now);
        let period_date = PeriodDate::Days(1);
        let utc_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(yesterday)
            .end_date_time(now)
            .period_date(period_date)
            .build()
            .unwrap();

        // Create Parameters
        let parameters: Parameters = Parameters {
            p_values: PSet::from_iter([
                P {
                    k: "t_2m",
                    v: Some("C"),
                },
                P {
                    k: "precip_1h",
                    v: Some("mm"),
                },
            ]),
        };

        // Create Locations
        let locations: Locations = Locations {
            coordinates: Coordinates::from(["47.419708", "9.358478"]),
        };

        // Call endpoint
        let result = meteomatics_connector
            .query_time_series(utc_vdt, parameters, locations, None)
            .await;

        match result {
            Ok(ref response) => {
                println!(">>>>>>>>>> ResponseBody:\n{}", response.response_body);
                assert_eq!(response.http_status_code, "200");
                assert_ne!(response.response_body.to_string(), "");
            }
            Err(ref connector_error) => {
                println!(">>>>>>>>>> ConnectorError: {:#?}", connector_error);
                assert!(result.is_err());
            }
        }

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn use_period_date_and_time_step_simultaneously() {
        println!("\n##### use_period_date_and_time_step_simultaneously:");

        // Create API connector
        let meteomatics_connector = MeteomaticsConnector::new(
            "python-community".to_string(),
            "Umivipawe179".to_string(),
            10,
        );

        // Create ValidDateTime
        let now = Utc::now();
        let yesterday = VDTOffset::Utc(now.clone() - Duration::days(1));
        let now = VDTOffset::Utc(now);
        let period_date = PeriodDate::Days(1);
        let time_step = PeriodTime::Hours(1);
        let utc_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(yesterday)
            .end_date_time(now)
            .period_date(period_date)
            .time_step(time_step)
            .build()
            .unwrap();

        // Create Parameters
        let parameters: Parameters = Parameters {
            p_values: PSet::from_iter([
                P {
                    k: "t_2m",
                    v: Some("C"),
                },
                P {
                    k: "precip_1h",
                    v: Some("mm"),
                },
            ]),
        };

        // Create Locations
        let locations: Locations = Locations {
            coordinates: Coordinates::from(["47.419708", "9.358478"]),
        };

        // Call endpoint
        let result = meteomatics_connector
            .query_time_series(utc_vdt, parameters, locations, None)
            .await;

        match result {
            Ok(_) => {}
            Err(ref connector_error) => {
                println!(">>>>>>>>>> ConnectorError: {:#?}", connector_error);
                assert!(result.is_err());
            }
        }

        assert!(!result.is_ok());
    }
}
