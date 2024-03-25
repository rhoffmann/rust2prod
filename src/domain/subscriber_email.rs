use std::fmt::Formatter;

use validator::validate_email;
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    /// returns a new instance of SubscriberEmail when the input is valid
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
        match validate_email(&email) {
            true => Ok(Self(email)),
            false => Err(format!("Invalid email: {}", email)),
        }
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rand::prelude::StdRng;
    use rand::SeedableRng;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_is_accepted(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email.0);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn email_without_at_symbol_is_rejected() {
        let email = "test.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_without_domain_is_rejected() {
        let email = "test@.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_without_username_is_rejected() {
        let email = "@test.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_with_multiple_at_symbols_is_rejected() {
        let email = "test@test@test.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_email_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
