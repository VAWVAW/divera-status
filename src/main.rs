use divera_status::{get_token, run, Arguments};

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    let token = get_token(&args)?;

    run(args, token).await?;

    Ok(())
}
