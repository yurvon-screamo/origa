pub fn validate_email(email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Введите email".to_string());
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("Некорректный формат email".to_string());
    }
    Ok(())
}

pub fn validate_credentials(email: &str, _password: &str) -> Result<(), String> {
    validate_email(email)?;
    Ok(())
}
