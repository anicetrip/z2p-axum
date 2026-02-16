use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if !s.validate_email() {
            return Err(format!("{s} is not a valid subscriber email."));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;

    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use proptest::prelude::*;
    use proptest::prelude::{Strategy, any};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    fn valid_email_strategy() -> impl Strategy<Value = String> {
        any::<()>().prop_map(|_| SafeEmail().fake())
    }

    proptest! {
        #[test]
        fn valid_emails_are_parsed_successfully(email in valid_email_strategy()) {
            assert!(SubscriberEmail::parse(email).is_ok());
        }
    }
}
