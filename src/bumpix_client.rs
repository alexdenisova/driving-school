use std::collections::HashMap;
use std::fmt::Display;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use color_eyre::{eyre::eyre, Result as AnyResult};
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use serde::Deserialize;
use serde_with::formats::Flexible;
use serde_with::TimestampSeconds;
use urlencoding::encode as urlencode;

#[derive(Debug)]
pub struct BumpixClient {
    client: Client,
    base_url: Url,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleResponse {
    pub it: u16,
    pub sa: Vec<u16>,
    #[serde(default)]
    pub time: HashMap<UnixTime, SlotResponse>,
    #[serde(default)]
    pub events: HashMap<UnixTime, Vec<[MidnightTime; 2]>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlotResponse {
    pub w: [MidnightTime; 2],
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Eq, Deserialize)]
pub struct UnixTime(#[serde_as(as = "TimestampSeconds<String, Flexible>")] DateTime<Utc>);

impl PartialEq for UnixTime {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::hash::Hash for UnixTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for UnixTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UnixTime(time) = &self;
        write!(f, "{}", time.timestamp())
    }
}

impl UnixTime {
    pub fn to_naive_date(&self) -> NaiveDate {
        let UnixTime(time) = &self;
        time.date_naive()
    }
    pub fn in_two_weeks() -> Self {
        let naive = NaiveDateTime::new(
            chrono::offset::Local::now().date_naive() + Duration::days(15),
            NaiveTime::default(),
        );
        UnixTime(Utc.from_utc_datetime(&naive))
    }
    pub fn add_day(&self) -> Self {
        let UnixTime(time) = &self;
        UnixTime(time.to_owned() + Duration::days(1))
    }
}

#[derive(Debug, Clone, Deserialize)]
/// Minutes after midnight
pub struct MidnightTime(pub u16);

impl Display for MidnightTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let MidnightTime(time) = &self;
        write!(f, "{}", time)
    }
}

impl From<NaiveTime> for MidnightTime {
    fn from(value: NaiveTime) -> Self {
        MidnightTime((value.hour() * 60 + value.minute()) as u16)
    }
}

impl BumpixClient {
    pub fn new(base_url: &Url, phone_number: &str, password: &str) -> AnyResult<Self> {
        let mut headers = HeaderMap::new();
        let cookie = BumpixClient::get_cookie(base_url, phone_number, password)?;
        headers.insert(header::COOKIE, HeaderValue::from_str(&cookie)?);

        let client = ClientBuilder::new().default_headers(headers).build()?;
        Ok(Self {
            client,
            base_url: base_url.to_owned(),
        })
    }

    fn get_cookie(base_url: &Url, phone_number: &str, password: &str) -> AnyResult<String> {
        let client = Client::new();
        let response = client
            .post(base_url.join("/data/api/site_login")?)
            .body(format!(
                "p={}&t={}",
                urlencode(password),
                urlencode(phone_number)
            ))
            .send()?
            .error_for_status()?;
        log::debug!("Sign-in response:\n{:?}", response);
        let headers = response.headers();
        let re = Regex::new(r"(PHPSESSID=\S+);.*").unwrap();
        for cookie in headers.get_all("Set-Cookie") {
            if let Some(caps) = re.captures(cookie.to_str()?) {
                return Ok(caps.get(1).unwrap().as_str().to_owned());
            }
        }
        Err(eyre!("No cookie found in response headers"))
    }

    pub fn get_schedule(
        &self,
        instructor_id: u32,
        start_date: &UnixTime,
        end_date: &UnixTime,
    ) -> AnyResult<ScheduleResponse> {
        let response = self
            .client
            .post(
                self.base_url
                    .join("/data/api/site_get_data_for_appointment")?,
            )
            .body(format!(
                "generalId={}&insideId=1.1&from={}&to={}&teid=-1",
                instructor_id, start_date, end_date
            ))
            .send()?
            .error_for_status()?
            .json::<ScheduleResponse>()?;
        log::debug!("Get schedule response:\n{:?}", response);
        Ok(response)
    }

    pub fn post_appointment(
        &self,
        instructor_id: u32,
        date: &UnixTime,
        time: &MidnightTime,
    ) -> AnyResult<()> {
        self.client
            .post(self.base_url.join("/data/api/site_appointment")?)
            .body(format!(
                "uid={}&mid=1.1&s=1.1%2C&sc=1%2C&d={}&t={}&te=-1&non=&nop=&oc=",
                instructor_id, date, time
            ))
            .send()?
            .error_for_status()?;
        Ok(())
    }
}
