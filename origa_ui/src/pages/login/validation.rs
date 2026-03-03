const MIN_PASSWORD_LENGTH: usize = 8;

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

pub fn validate_password(password: &str) -> Result<(), String> {
    let password = password.trim();
    if password.is_empty() {
        return Err("Введите пароль".to_string());
    }
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Пароль должен быть минимум {} символов",
            MIN_PASSWORD_LENGTH
        ));
    }
    Ok(())
}

pub fn validate_credentials(email: &str, password: &str) -> Result<(), String> {
    validate_email(email)?;
    validate_password(password)?;
    Ok(())
}
