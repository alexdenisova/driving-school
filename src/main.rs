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
    let schedule = Schedule::from_response(client.get_schedule(cli.instructor_id(), &start_time, &end_time)?);

    if schedule.time_is_free(&cli.time) {
        client.post_appointment(cli.instructor_id(), &start_time, &cli.time.into())?;
        log::info!("Created appointment")
    } else {
        log::error!("Could not sign up, {} was not free", cli.time);
    }
    Ok(())
}
