use chrono::NaiveTime;
use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use schedule::Schedule;
use settings::Cli;

use crate::bumpix_client::UnixTime;

mod bumpix_client;
mod schedule;
mod settings;

fn main() -> AnyResult<()> {
    dotenv()?;
    let cli = Cli::parse();
    cli.setup_logging()?;

    let client = cli.bumpix_client()?;
    let start_time = UnixTime::in_two_weeks();
    let end_time = start_time.add_day();
    let schedule = Schedule::from_response(client.get_schedule(&start_time, &end_time)?);

    if schedule.time_is_free(&NaiveTime::from_hms_opt(9, 0, 0).unwrap()) {
        log::info!("Time is free")
    } else {
        log::error!("Could not sign up, {} was not free", cli.time);
    }
    Ok(())
}
