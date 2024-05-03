use boxsy::{run, window_conf};
use boxsy_ceptre::{generate, Error};

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Error<'static>> {
    let game = generate(std::path::Path::new("crawler.json"))?;
    run(game).await;
    Ok(())
}
