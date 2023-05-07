use crate::connector_error::ConnectorError;
use crate::connector_response::{ConnectorResponse, ResponseBody};
use crate::format::Format;
use crate::locations::Locations;
use crate::optionals::Optionals;
use crate::parameters::Parameters;
use crate::valid_date_time::ValidDateTime;
use reqwest::{Client, Response, StatusCode};
use url::{ParseError, Url};

const DEFAULT_API_BASE_URL: &str = "https://api.meteomatics.com";

#[derive(Clone, Debug)]
pub struct APIClient {
    http_client: Client,
    username: String,
    password: String,
}

impl APIClient {
    pub fn new(username: String, password: String, timeout_seconds: u64) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .build()
            .unwrap();
        Self {
            http_client,
            username,
            password,
        }
    }

    pub async fn query_time_series(
        &self,
        vdt: ValidDateTime,
        parameters: Parameters<'_>,
        locations: Locations<'_>,
        optionals: Option<Optionals<'_>>,
    ) -> Result<ConnectorResponse, ConnectorError> {
        let url_fragment = match optionals {
            None => {
                format!(
                    "{}/{}/{}/{}",
                    vdt.format()?,
                    parameters,
                    locations,
                    Format::CSV
                )
            }
            Some(_) => {
                format!(
                    "{}/{}/{}/{}?{}",
                    vdt.format()?,
                    parameters,
                    locations,
                    Format::CSV,
                    optionals.unwrap()
                )
            }
        };

        let result = self.do_http_get(&url_fragment).await;

        match result {
            Ok(response) => match response.status() {
                StatusCode::OK => {
                    let prefix_headers = vec!["validdate".to_string()];
                    let connector_response: ConnectorResponse = self
                        .create_response(response, prefix_headers, parameters)
                        .await?;
                    Ok(connector_response)
                }
                status => Err(ConnectorError::HttpError(
                    status.to_string(),
                    response.text().await.unwrap(),
                    status,
                )),
            },
            Err(connector_error) => Err(ConnectorError::ApiError {
                source: connector_error,
            }),
        }
    }

    async fn do_http_get(&self, url_fragment: &str) -> Result<Response, reqwest::Error> {
        let full_url = build_url(url_fragment)
            .await
            .expect("URL fragment must be valid");

        println!(">>>>>>>>>> full_url: {}", full_url);

        self.http_client
            .get(full_url)
            .basic_auth(&self.username, Some(String::from(&self.password)))
            .send()
            .await
    }

    async fn create_response(
        &self,
        response: Response,
        prefix_headers: Vec<String>,
        parameters: Parameters<'_>,
    ) -> Result<ConnectorResponse, ConnectorError> {
        let status = response.status();
        // println!(">>>>>>>>>> reqwest status: {}", status);
        // println!(">>>>>>>>>> reqwest headers:\n{:#?}", response.headers());

        let body = response.text().await.unwrap();
        // println!(">>>>>>>>>> reqwest body:\n{}", body);

        let mut response_body: ResponseBody = ResponseBody::new();
        for header in prefix_headers {
            response_body.add_header(header);
        }
        let p_values = parameters.p_values;
        for p_value in p_values.clone() {
            response_body.add_header(p_value.to_string());
        }

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(body.as_bytes());

        let result_body = response_body
            .populate_records(&mut rdr, p_values.len())
            .await
            .map_err(|error| ConnectorError::GenericError(error));
        // println!(">>>>>>>>>> result body:\n{}", result_body);

        match result_body {
            Ok(_) => Ok(ConnectorResponse {
                response_body,
                http_status_code: status.as_str().to_string(),
                http_status_message: status.to_string(),
            }),
            Err(connector_error) => Err(connector_error),
        }
    }
}

async fn build_url(url_fragment: &str) -> Result<Url, ParseError> {
    let base_url = Url::parse(DEFAULT_API_BASE_URL).expect("Base URL is known to be valid");
    let full_url = base_url.join(url_fragment)?;
    Ok(full_url)
}

#[cfg(test)]
mod tests {

    use crate::configuration::api_client::APIClient;
    use crate::connector_components::format::Format;
    use crate::entities::connector_response::ResponseBody;
    use crate::locations::{Coordinates, Locations};
    use crate::parameters::{PSet, Parameters, P};
    use crate::valid_date_time::{PeriodTime, VDTOffset, ValidDateTime, ValidDateTimeBuilder};
    use chrono::{Duration, Local};
    use reqwest::StatusCode;
    use std::iter::FromIterator;

    #[tokio::test]
    async fn client_fires_get_request_to_base_url() {
        println!("\n##### client_fires_get_request_to_base_url:");

        // Change to correct username and password.
        let api_client = APIClient::new(
            "python-community".to_string(),
            "Umivipawe179".to_string(),
            10,
        );
        println!(">>>>>>>>>> api_client: {:?}", api_client);

        let now = Local::now();
        let yesterday = now.clone() - Duration::days(1);
        println!(">>>>>>>>>> yesterday (local) {:?}", yesterday);
        println!(">>>>>>>>>> now (local) {:?}", now);
        let yesterday = VDTOffset::Local(now.clone() - Duration::days(1));
        let now = VDTOffset::Local(now);
        let time_step = PeriodTime::Hours(1);
        let local_vdt: ValidDateTime = ValidDateTimeBuilder::default()
            .start_date_time(yesterday)
            .end_date_time(now)
            .time_step(time_step)
            .build()
            .unwrap();

        // Create Parameters
        let parameters: Parameters = Parameters {
            p_values: PSet::from_iter([P {
                k: "t_2m",
                v: Some("C"),
            }]),
        };

        // Create Locations
        let locations: Locations = Locations {
            coordinates: Coordinates::from(["52.520551", "13.461804"]),
        };

        let url_fragment = &*format!(
            "{}--{}{}/{}/{}/{}",
            local_vdt.start_date_time,
            local_vdt.end_date_time.unwrap(),
            ":".to_string() + &*time_step.to_string(),
            parameters,
            locations,
            Format::CSV.to_string()
        );
        println!(">>>>>>>>>> url_fragment: {:?}", url_fragment);

        let result = api_client.do_http_get(url_fragment).await;
        // println!("response: {:?}", response);

        match result {
            Ok(response) => match response.status() {
                StatusCode::OK => {
                    let status = response.status();
                    println!(">>>>>>>>>> reqwest status: {}", status);
                    // println!(">>>>>>>>>> reqwest headers:\n{:#?}", response.headers());

                    let body = response.text().await.unwrap();
                    println!(">>>>>>>>>> reqwest body:\n{}", body);

                    let mut response_body: ResponseBody = ResponseBody::new();
                    let p_values = parameters.p_values;
                    for p_value in p_values.clone() {
                        response_body.add_header(p_value.to_string());
                    }

                    let mut rdr = csv::ReaderBuilder::new()
                        .delimiter(b';')
                        .from_reader(body.as_bytes());
                    response_body
                        .populate_records(&mut rdr, p_values.len())
                        .await
                        .unwrap();
                    println!(">>>>>>>>>> ResponseBody:\n{}", response_body);

                    print!(">>>>>>>>>> ResponseHeaders:\n");
                    println!("{}", response_body.response_headers.to_vec().join(","));

                    print!("\n>>>>>>>>>> ResponseRecords:\n");
                    for response_record in response_body.response_records {
                        println!("{:#?}", response_record);
                    }

                    assert_eq!(status.as_str(), "200");
                    assert_ne!(body, "");
                }
                status => {
                    println!(">>>>>>>>>> StatusCode error: {:#?}", status.to_string());
                    assert_eq!(status.as_str(), "200"); // Assert to fail
                }
            },
            Err(ref error) => {
                println!(">>>>>>>>>> error: {:#?}", error);
                assert!(result.is_ok());
            }
        }
    }
}
