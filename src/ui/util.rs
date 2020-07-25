use anyhow::Result;
use iced::image;

pub async fn fetch_image(url: String) -> Result<image::Handle> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let bytes = reqwest::get(&url).await?.bytes().await?;

        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
    }

    #[cfg(target_arch = "wasm32")]
    Ok(image::Handle::from_path(url))
}