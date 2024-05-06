use boxsy::{run, window_conf};
use boxsy_ceptre::{generate, Error};

use std::env;

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Error<'static>> {
    let arg = env::args()
        .nth(1)
        .ok_or(Error::CommandLine("file not given"))?;
    let game = generate(std::path::PathBuf::from(arg))?;
    run(game).await;
    Ok(())
}
