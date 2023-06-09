use image::{io::Reader as ImageReader, DynamicImage};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, io::Cursor};

use anyhow::{anyhow, Result};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "https://api.unsplash.com";

fn unsplash_reqwest_client(access_key: &str) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert("Accept-Version", HeaderValue::from_static("v1"));
    headers.insert(
        header::AUTHORIZATION,
        format!("Client-ID {access_key}")
            .try_into()
            .expect("access_key should not contain invalid header characters"),
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("reqwest client should build successfully")
}

#[derive(Serialize, Deserialize)]
pub struct GetRandomPhotoOptions {
    pub collections: Option<String>,
    pub topics: Option<String>,
    pub username: Option<String>,
    pub orientation: Option<Orientation>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
    Squarish,
}

// TODO: implement proper error handling
pub async fn get_random_photo(
    access_key: &str,
    options: GetRandomPhotoOptions,
) -> Result<DynamicImage> {
    let metadata: HashMap<String, JsonValue> = unsplash_reqwest_client(access_key)
        .get(format!("{API_BASE_URL}/photos/random"))
        .query(&options)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let url = metadata
        .get("urls")
        .ok_or(anyhow!("Metadata missing `urls` key"))?
        .get("regular")
        .ok_or(anyhow!("Metadata missing `urls.regular` key"))?
        .as_str()
        .ok_or(anyhow!("Invalid data type of `urls.regular` metadata key"))?;

    let image_data = reqwest::get(url).await?.error_for_status()?.bytes().await?;

    let image = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()?
        .decode()?;

    Ok(image)
}
