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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ImageParseError;

    #[test]
    fn test_parse_image_tag_with_image_and_tag() {
        let input = "myimage:v1".to_string();
        let result = parse_image_tag(input).unwrap();
        assert_eq!(result, "myimage:v1");
    }

    #[test]
    fn test_parse_image_tag_with_image_only() {
        let input = "myimage".to_string();
        let result = parse_image_tag(input).unwrap();
        assert_eq!(result, "myimage:latest");
    }

    #[test]
    fn test_parse_image_tag_with_empty_string() {
        let input = "".to_string();
        let result = parse_image_tag(input);
        assert!(matches!(result, Err(ImageParseError::EmptyImage)));
    }

    #[test]
    fn test_parse_image_tag_with_empty_image_part() {
        let input = ":v1".to_string();
        let result = parse_image_tag(input);
        assert!(matches!(result, Err(ImageParseError::EmptyPart(ref s)) if s == ":v1"));
    }

    #[test]
    fn test_parse_image_tag_with_empty_tag_part() {
        let input = "myimage:".to_string();
        let result = parse_image_tag(input);
        assert!(matches!(result, Err(ImageParseError::EmptyPart(ref s)) if s == "myimage:"));
    }
}
