fn mc_to_ansi_char<'a>(mc: char) -> &'a str {
    match mc {
        // color codes
        '0' => "30",
        '1' => "34",
        '2' => "32",
        '3' => "36",
        '4' => "31",
        '5' => "35",
        '6' => "33",
        '7' => "37",
        '8' => "30",
        '9' => "34",
        'a' => "32",
        'b' => "36",
        'c' => "31",
        'd' => "35",
        'e' => "33",
        'f' => "37",

        // escape codes
        'l' => "1",
        'm' => "4",
        'n' => "4",
        'r' => "0",

        _ => "0",
    }
}

pub fn mc_to_ansi<T: ToString>(text: T) -> String {
    let text = text.to_string();
    let mut built_string = String::with_capacity(text.len());

    let mut chars = text.chars();
    loop {
        let Some(ch) = chars.next() else {
            return built_string;
        };
        if ch == '\u{A7}' {
            let Some(code) = chars.next() else {
                return built_string;
            };
            built_string.push_str(&format!("\u{1b}[{}m", mc_to_ansi_char(code)));
        } else {
            built_string.push(ch);
        }
    }
}
