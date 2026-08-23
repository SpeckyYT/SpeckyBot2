pub fn uppercase_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

#[inline]
pub fn code_block(content: impl AsRef<str>, language: impl AsRef<str>) -> String {
    let content = content.as_ref();
    let language = language.as_ref();

    let mut code = String::with_capacity(4 * 2 + 16 + language.len() + content.len());
    code.push_str("```");
    code.push_str(language);
    code.push('\n');
    for (i, part) in content.split("```").enumerate() {
        if i > 0 { code.push_str("`\u{200B}``"); }
        code.push_str(part);
    }
    code.push_str("\n```");
    code
}
