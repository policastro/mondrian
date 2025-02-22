use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Asset;

impl Asset {
    pub fn get_string(path: &str) -> Result<String, std::io::Error> {
        let asset = Asset::get(path).ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Asset not found"))?;
        std::str::from_utf8(asset.data.as_ref())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            .map(|s| s.to_string())
    }
}
