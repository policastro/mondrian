use {
    std::{env, io, path::PathBuf},
    winresource::WindowsResource,
};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new().set_icon(PathBuf::new().join("assets").join("mondrian.ico").to_str().unwrap()).compile()?;
    }
    Ok(())
}
