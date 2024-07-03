use crate::error::ImageParseError;

pub fn parse_image_tag(input: String) -> Result<String, ImageParseError> {
    if input.is_empty() {
        return Err(ImageParseError::EmptyImage);
    }

    match input.split_once(':') {
        Some(("", _)) | Some((_, "")) => Err(ImageParseError::EmptyPart(input)),
        Some((image, tag)) => Ok(format!("{}:{}", image, tag)),
        None => Ok(format!("{}:latest", input)),
    }
}
