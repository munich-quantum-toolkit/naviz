use std::{
    env,
    error::Error,
    fs::File,
    io::{self},
    path::Path,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    download_default_font()?;

    Ok(())
}

/// Default font: FiraMono
/// License: OFL-1.1
const DEFAULT_FONT_URL: &str =
    "https://github.com/mozilla/Fira/raw/refs/heads/master/ttf/FiraMono-Regular.ttf";

/// Download the default-font (if it does not yet exist)
/// and set the `DEFAULT_FONT_PATH` environment-variable for the build
fn download_default_font() -> Result<(), Box<dyn Error>> {
    let font_out = Path::new(&env::var("OUT_DIR")?).join("default-font.ttf");

    if !font_out.exists() {
        let font = ureq::get(DEFAULT_FONT_URL).call()?;
        io::copy(
            &mut font.into_body().into_reader(),
            &mut File::create(&font_out)?,
        )?;
    }

    println!(
        "cargo::rustc-env=DEFAULT_FONT_PATH={}",
        font_out.to_string_lossy()
    );

    Ok(())
}
