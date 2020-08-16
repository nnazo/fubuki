use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "res/"]
pub struct Resources;
