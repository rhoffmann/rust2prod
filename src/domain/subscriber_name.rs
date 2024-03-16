
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// returns a new instance of SubscriberName when the input is valid
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '&', ':', ';', '@', ',', '.'];
        let contains_forbidden_characters = name.chars().any(|char| forbidden_characters.contains(&char));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("Invalid subscriber name: {}", name))
        } else {
            Ok(Self(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_ok, assert_err};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "g̈".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "g̈".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_name_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_forbidden_characters_is_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '&', ':', ';', '@', ',', '.'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
}