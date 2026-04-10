fn is_fuzzy_match(text: &str, query_lower: &str) -> bool {
    let mut query_chars = query_lower.chars().peekable();
    if query_chars.peek().is_none() {
        return true;
    }
    for c in text.chars() {
        let mut matched = false;
        if c.is_ascii() {
            if Some(&c.to_ascii_lowercase()) == query_chars.peek() {
                matched = true;
            }
        } else {
            for lc in c.to_lowercase() {
                if Some(&lc) == query_chars.peek() {
                    matched = true;
                    break;
                }
            }
        }

        if matched {
            query_chars.next();
            if query_chars.peek().is_none() {
                return true;
            }
        }
    }
    false
}

fn main() {
    assert!(is_fuzzy_match("hello world", "lo ld"));
    println!("Tests passed!");
}
