//! Application assets:
//! - Server icons

use std::io::Write;

use image::{DynamicImage, GenericImageView};

type OptIcon = Option<String>;

/// Server icon assets.
#[derive(Clone)]
pub struct ServerIcons {
    pub no_hosts: OptIcon,
    pub many_hosts: OptIcon,
}

impl ServerIcons {
    const NO_HOSTS: &'static str = "assets/icons/NoHosts.png";
    const MANY_HOSTS: &'static str = "assets/icons/ManyHosts.png";

    /// Encode image in base 64.
    fn encode(image: DynamicImage) -> OptIcon {
        let mut png: Vec<u8> = Vec::new();
        image.write_to(&mut png, image::ImageOutputFormat::Png).expect("Failed to encode image as PNG.");
        let mut encoder = base64::write::EncoderStringWriter::new(base64::STANDARD_NO_PAD);
        encoder.write_all(&png).expect("Failed to encode image in base 64.");
        return Some(encoder.into_inner())
    }

    /// Validate image with server icon requirements. (64 x 64)
    /// Return the image encoded.
    fn encode_valid(name: String, image: DynamicImage) -> OptIcon {
        if image.dimensions() == (64, 64) {
            return Self::encode(image)
        }
        println!("Asset {} does not match the dimensions. (64x64 pixels)", name);
        None
    }

    /// Load, validate and encode image.
    fn get_icon(name: String, path: &String) -> OptIcon {
        let result = image::open(path);
        if let Ok(image) = result {
            return Self::encode_valid(name, image)
        }
        println!("Asset {} does not exist in '{}' or has invalid format.", name, path);
        None
    }

    /// Load all icons.
    pub fn get_icons() -> Self {
        Self {
            no_hosts: Self::get_icon("'No hosts' image".to_string(),  &ServerIcons::NO_HOSTS.to_string()),
            many_hosts: Self::get_icon("'Many hosts' image".to_string(), &ServerIcons::MANY_HOSTS.to_string()),
        }
    }
}
