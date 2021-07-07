/*
For loading and validating images and turning them into base64 server icons.
*/

use std::io::Write;

use image::{DynamicImage, GenericImageView};

type OptIcon = Option<String>;

pub struct ServerIcons {
    pub no_hosts: OptIcon,
    pub many_hosts: OptIcon,
}

impl ServerIcons {
    const NO_HOSTS: &'static str = "NoHosts.png";
    const MANY_HOSTS: &'static str = "ManyHosts.png";

    /*
    Loads raw image into memory.
    */
    fn load(path: &String) -> Option<DynamicImage> {
        let result = image::open(path);
        if let Ok(image) = result {
            return Some(image)
        }
        None
    }

    /*
    Validates image with server icon requirements. (64 x 64)
    */
    fn validate(opt_image: Option<DynamicImage>) -> Option<DynamicImage> {
        if let Some(image) = opt_image {
            if image.dimensions() == (64, 64) {
                return Some(image)
            }
        }
        None
    }

    /*
    Encodes the image in b64.
    */
    fn encode(opt_image: Option<DynamicImage>) -> Option<String> {
        if let Some(image) = opt_image {
            let mut png: Vec<u8> = Vec::new();
            image.write_to(&mut png, image::ImageOutputFormat::Png).unwrap();
            let mut encoder = base64::write::EncoderStringWriter::new(base64::STANDARD_NO_PAD);
            encoder.write_all(&png).expect("Failed to encode image in base 64.");
            return Some(encoder.into_inner())
        }
        None
    }

    /*
    Loads, validates and encodes one image as icon.
    */
    fn get_icon(path: &String) -> Option<String> {
        let opt_image = Self::load(path);
        let opt_image = Self::validate(opt_image);
        Self::encode(opt_image)
    }

    /*
    Loads all images into an icon list.
    */
    pub fn get_icons() -> Self {
        Self {
            no_hosts: Self::get_icon(&ServerIcons::NO_HOSTS.to_string()),
            many_hosts: Self::get_icon(&ServerIcons::MANY_HOSTS.to_string()),
        }
    }
}
