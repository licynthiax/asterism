use boxsy_ceptre::{generate, Error};

fn main() -> Result<(), Error<'static>> {
    let _game = generate(std::path::Path::new("crawler.json"))?;
    Ok(())
}
