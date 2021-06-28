use eyre::Result;
use memeroute_gui::run;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    color_eyre::install()?;
    run().await
}
