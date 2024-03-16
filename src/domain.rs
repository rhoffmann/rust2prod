use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    /// returns a new instance of SubscriberName when the input is valid
    /// # Panics
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '&', ':', ';', '@', ',', '.'];
        let contains_forbidden_characters = name.chars().any(|char| forbidden_characters.contains(&char));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("Invalid subscriber name {}", name))
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