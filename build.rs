use {
    std::{env, io, path::PathBuf},
    winresource::WindowsResource,
};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let icon_path = PathBuf::new().join("./assets").join("mondrian.ico");
        WindowsResource::new()
            .set_icon_with_id(icon_path.clone().to_str().unwrap(), "APP_ICON")
            .compile()?;
    }
    Ok(())
}
