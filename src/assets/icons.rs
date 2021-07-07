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
    const NO_HOSTS: &'static str = "assets/icons/NoHosts.png";
    const MANY_HOSTS: &'static str = "assets/icons/ManyHosts.png";

    /*
    Encodes the image in b64.
    */
    fn encode(image: DynamicImage) -> Option<String> {
        let mut png: Vec<u8> = Vec::new();
        image.write_to(&mut png, image::ImageOutputFormat::Png).expect("Failed to encode image as PNG.");
        let mut encoder = base64::write::EncoderStringWriter::new(base64::STANDARD_NO_PAD);
        encoder.write_all(&png).expect("Failed to encode image in base 64.");
        return Some(encoder.into_inner())
    }

    /*
    Validates image with server icon requirements. (64 x 64)
    Returns the encoded image.
    */
    fn encode_valid(name: String, image: DynamicImage) -> Option<String> {
        if image.dimensions() == (64, 64) {
            return Self::encode(image)
        }
        println!("Asset {} does not match the dimensions. (64x64 pixels)", name);
        None
    }

    /*
    Loads, validates and encodes one image as icon.
    */
    fn get_icon(name: String, path: &String) -> Option<String> {
        let result = image::open(path);
        if let Ok(image) = result {
            return Self::encode_valid(name, image)
        }
        println!("Asset {} does not exist in {} or has invalid format.", name, path);
        None
    }

    /*
    Loads all images into an icon list.
    */
    pub fn get_icons() -> Self {
        Self {
            no_hosts: Self::get_icon("'No hosts' image".to_string(),  &ServerIcons::NO_HOSTS.to_string()),
            many_hosts: Self::get_icon("'Many hosts' image".to_string(), &ServerIcons::MANY_HOSTS.to_string()),
        }
    }
}
