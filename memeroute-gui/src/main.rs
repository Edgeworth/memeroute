use eyre::Result;
use memeroute_gui::run;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    color_eyre::install()?;
    run().await
}
