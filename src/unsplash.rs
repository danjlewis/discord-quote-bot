use std::{collections::HashMap, io::Cursor};

use anyhow::{anyhow, Context, Result};
use image::{io::Reader as ImageReader, RgbImage};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, IntoUrl, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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
                .expect("Unsplash access key should not contain invalid HTTP header characters"),
        );

        Client::builder()
            .default_headers(headers)
            .build()
            .expect("Unsplash reqwest client should build successfully")
    }

    async fn download_from_raw_url(
        &self,
        url: impl IntoUrl,
        imgix_params: &ImgixParams,
    ) -> Result<RgbImage> {
        let image_data = self
            .reqwest_client
            .get(url)
            .query(&imgix_params)
            .send()
            .await
            .context("error occurred while sending request")?
            .error_for_status()
            .context("image download request failed")?
            .bytes()
            .await
            .context("invalid image download response format")?;

        let image = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .context("failed to guess image format")?
            .decode()
            .context("failed to decode image")?
            .into_rgb8();

        Ok(image)
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
            .await
            .context("error occurred while sending request")?
            .error_for_status()
            .context("random photo request failed")?
            .json()
            .await
            .context("invalid random photo response format")?;

        let url: Url = metadata
            .get("urls")
            .ok_or(anyhow!("metadata missing `urls` key"))?
            .get("raw")
            .ok_or(anyhow!("metadata missing `urls.raw` key"))?
            .as_str()
            .ok_or(anyhow!("invalid data type of `urls.raw` metadata key"))?
            .parse()
            .context("invalid URL format provided by Unsplash API")?;

        let image = self
            .download_from_raw_url(url, &options.imgix_params)
            .await
            .context("failed to download image")?;

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
    pub aspect_ratio: Option<String>,
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
