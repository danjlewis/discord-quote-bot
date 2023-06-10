use image::{io::Reader as ImageReader, DynamicImage};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, io::Cursor};

use anyhow::{anyhow, Result};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Url,
};
use serde::{Deserialize, Serialize};

pub struct UnsplashClient {
    reqwest_client: Client,
    unsplash_reqwest_client: Client,
    unsplash_base_url: Url,
}

impl UnsplashClient {
    pub fn new(access_key: &str) -> Self {
        let reqwest_client = Client::builder()
            .build()
            .expect("reqwest client should build successfully");

        let unsplash_reqwest_client = Self::unsplash_reqwest_client(access_key);

        let unsplash_base_url = Url::parse("https://api.unsplash.com")
            .expect("Unsplash base URL should parse successfully");

        Self {
            reqwest_client,
            unsplash_reqwest_client,
            unsplash_base_url,
        }
    }

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
            .expect("Unsplash reqwest client should build successfully")
    }

    // TODO: implement proper error handling
    pub async fn get_random_photo(&self, options: GetRandomPhotoOptions) -> Result<DynamicImage> {
        let metadata: HashMap<String, JsonValue> = self
            .unsplash_reqwest_client
            .get(
                self.unsplash_base_url
                    .join("/photos/random")
                    .expect("Unsplash random photo endpoint URL should parse correctly"),
            )
            .query(&options)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let raw_url = metadata
            .get("urls")
            .ok_or(anyhow!("Metadata missing `urls` key"))?
            .get("raw")
            .ok_or(anyhow!("Metadata missing `urls.raw` key"))?
            .as_str()
            .ok_or(anyhow!("Invalid data type of `urls.raw` metadata key"))?;

        let url = format!("{raw_url}?w=1000&h=1000&ar=1:1&fit=crop");

        let image_data = self
            .reqwest_client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let image = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()?
            .decode()?;

        Ok(image)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
    Squarish,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct GetRandomPhotoOptions {
    pub collections: Option<String>,
    pub topics: Option<String>,
    pub username: Option<String>,
    pub orientation: Option<Orientation>,
}
