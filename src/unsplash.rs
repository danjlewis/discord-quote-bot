use image::{io::Reader as ImageReader, RgbImage};
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
    pub async fn get_random_photo(&self, options: GetRandomPhotoOptions) -> Result<RgbImage> {
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

        let url: Url = metadata
            .get("urls")
            .ok_or(anyhow!("Metadata missing `urls` key"))?
            .get("raw")
            .ok_or(anyhow!("Metadata missing `urls.raw` key"))?
            .as_str()
            .ok_or(anyhow!("Invalid data type of `urls.raw` metadata key"))?
            .parse()?;

        let image_data = self
            .reqwest_client
            .get(url)
            .query(&options.imgix_params)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let image = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()?
            .decode()?
            .into_rgb8();

        Ok(image)
    }

    pub async fn generate_background_image(&self) -> Result<RgbImage> {
        let api_options = GetRandomPhotoOptions {
            collections: Some(String::from("11649432")),
            imgix_params: ImgixParams {
                width: Some(1000),
                height: Some(1000),
                format: Some(ImgixFormat::Jpg),
                quality: Some(45),
                fit_mode: Some(ImgixFitMode::Crop),
                aspect_ratio: Some([1, 1]),
            },
            ..Default::default()
        };
        let image = self.get_random_photo(api_options).await?;

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
pub struct ImgixParams {
    #[serde(rename = "fm")]
    pub format: Option<ImgixFormat>,
    #[serde(rename = "w")]
    pub width: Option<u32>,
    #[serde(rename = "h")]
    pub height: Option<u32>,
    #[serde(rename = "q")]
    pub quality: Option<u32>,
    #[serde(rename = "fit")]
    pub fit_mode: Option<ImgixFitMode>,
    #[serde(rename = "ar")]
    pub aspect_ratio: Option<[u32; 2]>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ImgixFormat {
    Png,
    Jpg,
    Json,
    WebP,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ImgixFitMode {
    Clamp,
    Clip,
    Crop,
    FaceArea,
    Fill,
    FillMax,
    Max,
    Min,
    Scale,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct GetRandomPhotoOptions {
    pub collections: Option<String>,
    pub topics: Option<String>,
    pub username: Option<String>,
    pub orientation: Option<Orientation>,
    #[serde(skip)]
    pub imgix_params: ImgixParams,
}
