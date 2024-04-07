use std::str::FromStr;

use clap::{Args, Parser};
use color_eyre::eyre::eyre;
use color_eyre::Result as AnyResult;

use chrono::prelude::*;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter;
use regex::Regex;
use reqwest::Url;

use crate::bumpix_client::BumpixClient;

/// Bumpix lesson sign-up utility
#[derive(Debug, Parser)]
pub struct Cli {
    /// Time to sign up for lessons
    #[arg(long, short = 't', value_parser = parse_time, env = "DS__SIGN_UP_TIME")]
    pub time: NaiveTime,
    /// Bumpix paramaters
    #[command(flatten)]
    bumpix: BumpixArguments,
    #[arg(long, short = 'd', default_value = "false", env = "DS__DEBUG")]
    debug: bool,
}

fn parse_time(arg: &str) -> AnyResult<NaiveTime> {
    let re = Regex::new(r"([0-9]{1,2}):([0-9]{1,2})").unwrap();
    let caps = re
        .captures(arg)
        .ok_or(eyre!("Wrong time format, should be 00:00"))?;
    let hour = caps.get(1).unwrap().as_str().parse()?;
    let min = caps.get(2).unwrap().as_str().parse()?;
    NaiveTime::from_hms_opt(hour, min, 0).ok_or(eyre!("{} is incorrect time", arg))
}

#[derive(Debug, Args)]
pub struct BumpixArguments {
    /// Phone number to sign in
    #[arg(long, short = 'n', env = "DS__PHONE_NUMBER")]
    phone_number: String,
    /// Password to sign in
    #[arg(long, short = 'p', env = "DS__PASSWORD")]
    password: String,
    /// Driving instructor id
    #[arg(long, env = "DS__INSTRUCTOR_ID")]
    instructor_id: u32,
}

impl Cli {
    pub fn setup_logging(&self) -> AnyResult<()> {
        let colors = ColoredLevelConfig::new()
            .debug(Color::BrightBlack)
            .info(Color::BrightGreen)
            .warn(Color::BrightYellow)
            .error(Color::BrightRed);
        Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{} [{}] {}",
                    Utc::now()
                        .with_timezone(&chrono_tz::Europe::Moscow)
                        .format("%Y-%m-%d %H:%M:%S MSK"),
                    colors.color(record.level()),
                    message
                ));
            })
            .level(if self.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            })
            .chain(std::io::stderr())
            .apply()?;
        Ok(())
    }
    pub fn bumpix_client(&self) -> AnyResult<BumpixClient> {
        BumpixClient::new(
            &Url::from_str("https://bumpix.net")?,
            &self.bumpix.phone_number,
            &self.bumpix.password,
        )
    }
    pub fn instructor_id(&self) -> u32 {
        self.bumpix.instructor_id
    }
}
