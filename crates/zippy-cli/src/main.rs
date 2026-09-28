use color_eyre::Result;

mod args;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args::Args { device, action } = args::parse_args()?;
    Ok(())
}
