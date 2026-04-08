use crate::i18n::{I18nContext, Locale};

pub fn validate_email(i18n: &I18nContext<Locale>, email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err(i18n.get_keys().login().email_required().inner().to_string());
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(i18n.get_keys().login().email_required().inner().to_string());
    }
    Ok(())
}

pub fn validate_credentials(
    i18n: &I18nContext<Locale>,
    email: &str,
    _password: &str,
) -> Result<(), String> {
    validate_email(i18n, email)?;
    Ok(())
}
