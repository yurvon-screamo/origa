use regex::Regex;

lazy_static::lazy_static! {
    static ref RUBY_REGEX: Regex = Regex::new(r"<ruby>([^<]+)<rp>\(</rp><rt>([^<]+)</rt><rp>\)</rp></ruby>").unwrap();
}

pub fn format_japanese_text(text: &str) -> String {
    match origa::domain::furiganize_text(text) {
        Ok(furigana_html) => convert_ruby_to_text(&furigana_html),
        Err(_) => text.to_string(),
    }
}

fn convert_ruby_to_text(html: &str) -> String {
    RUBY_REGEX.replace_all(html, "$1($2)").to_string()
}
