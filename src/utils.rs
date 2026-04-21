pub fn split_camel_case(s: &str, separator: char) -> String {
    s.chars().enumerate().fold(String::new(), |mut acc, (i, c)| {
        if i > 0 && c.is_uppercase() {
            acc.push(separator);
        }
        acc.push(c);
        acc
    })
}