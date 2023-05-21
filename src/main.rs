use divera_status::{start, Arguments};

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    let token = args.get_token()?;
    start(args, token).await?;

    Ok(())
}
