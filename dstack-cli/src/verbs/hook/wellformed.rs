// verbs/hook/wellformed.rs
// Is the payload JSON at all? — jq's answer to that question, which is not serde_json's.

/// jq stops at 256 nested containers ("Exceeds depth limit for parsing"); serde_json stops at 128,
/// which is one of the two places the two parsers disagree about a document jq reads fine.
const DEPTH: usize = 256;

/// True when the text is one well-formed JSON value by jq's rules: a number of any magnitude, any
/// nesting up to jq's limit, and `\uXXXX` escapes where only an unpaired high surrogate is
/// refused (jq turns a lone low surrogate into U+FFFD and reads on).
///
/// The hook asks this only after serde_json has refused the payload, to tell "jq could not read it
/// either" — every field empty, which is what the shell's `jq ... 2>/dev/null || true` left behind
/// — from "jq read it and this build cannot", which is a block.
pub(super) fn is_json(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut at = 0;
    if !value(bytes, &mut at) {
        return false;
    }
    skip_ws(bytes, &mut at);
    at == bytes.len()
}

/// One value with everything inside it. The containers are walked with an explicit stack, so the
/// depth this scanner allows is jq's and not the recursion limit of whoever calls it.
fn value(b: &[u8], at: &mut usize) -> bool {
    let mut stack: Vec<bool> = Vec::new(); // true while the innermost container is an object
    loop {
        skip_ws(b, at);
        match b.get(*at) {
            Some(b'{') | Some(b'[') => {
                let object = b[*at] == b'{';
                *at += 1;
                if stack.len() >= DEPTH {
                    return false;
                }
                stack.push(object);
                skip_ws(b, at);
                let closed = match object {
                    true => b.get(*at) == Some(&b'}'),
                    false => b.get(*at) == Some(&b']'),
                };
                if closed {
                    *at += 1;
                    stack.pop();
                } else {
                    if object && !member(b, at) {
                        return false;
                    }
                    continue;
                }
            }
            Some(b'"') => {
                if !string(b, at) {
                    return false;
                }
            }
            Some(b't') => {
                if !word(b, at, b"true") {
                    return false;
                }
            }
            Some(b'f') => {
                if !word(b, at, b"false") {
                    return false;
                }
            }
            Some(b'n') => {
                if !word(b, at, b"null") {
                    return false;
                }
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                if !number(b, at) {
                    return false;
                }
            }
            _ => return false,
        }
        // After a value: close whatever it ended, or step to the next element or member.
        loop {
            skip_ws(b, at);
            let Some(object) = stack.last().copied() else {
                return true;
            };
            match b.get(*at) {
                Some(b',') => {
                    *at += 1;
                    if object && !member(b, at) {
                        return false;
                    }
                    break;
                }
                Some(b'}') if object => {
                    *at += 1;
                    stack.pop();
                }
                Some(b']') if !object => {
                    *at += 1;
                    stack.pop();
                }
                _ => return false,
            }
        }
    }
}

/// The `"key":` an object member opens with; the value after it is read by the caller.
fn member(b: &[u8], at: &mut usize) -> bool {
    skip_ws(b, at);
    if !string(b, at) {
        return false;
    }
    skip_ws(b, at);
    if b.get(*at) != Some(&b':') {
        return false;
    }
    *at += 1;
    true
}

fn word(b: &[u8], at: &mut usize, want: &[u8]) -> bool {
    if b.len() < *at + want.len() || &b[*at..*at + want.len()] != want {
        return false;
    }
    *at += want.len();
    true
}

/// JSON's grammar with no ceiling on the magnitude: 1e400 and a 400-digit integer are numbers jq
/// reads and serde_json refuses, which is the whole reason this scanner exists.
fn number(b: &[u8], at: &mut usize) -> bool {
    if b.get(*at) == Some(&b'-') {
        *at += 1;
    }
    match b.get(*at) {
        Some(b'0') => *at += 1,
        Some(c) if c.is_ascii_digit() => digits(b, at),
        _ => return false,
    }
    if b.get(*at) == Some(&b'.') {
        *at += 1;
        if !b.get(*at).is_some_and(u8::is_ascii_digit) {
            return false;
        }
        digits(b, at);
    }
    if matches!(b.get(*at), Some(b'e') | Some(b'E')) {
        *at += 1;
        if matches!(b.get(*at), Some(b'+') | Some(b'-')) {
            *at += 1;
        }
        if !b.get(*at).is_some_and(u8::is_ascii_digit) {
            return false;
        }
        digits(b, at);
    }
    true
}

fn digits(b: &[u8], at: &mut usize) {
    while b.get(*at).is_some_and(u8::is_ascii_digit) {
        *at += 1;
    }
}

/// A JSON string. An unpaired high surrogate is the one escape jq refuses; a lone low surrogate it
/// reads (as U+FFFD), so this scanner reads it too and the caller blocks on it.
fn string(b: &[u8], at: &mut usize) -> bool {
    if b.get(*at) != Some(&b'"') {
        return false;
    }
    *at += 1;
    loop {
        match b.get(*at) {
            None => return false,
            Some(b'"') => {
                *at += 1;
                return true;
            }
            Some(b'\\') => {
                *at += 1;
                match b.get(*at) {
                    Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => *at += 1,
                    Some(b'u') => {
                        let Some(first) = hex4(b, at) else {
                            return false;
                        };
                        if (0xD800..0xDC00).contains(&first) {
                            if b.get(*at) != Some(&b'\\') {
                                return false;
                            }
                            *at += 1;
                            if !hex4(b, at).is_some_and(|low| (0xDC00..0xE000).contains(&low)) {
                                return false;
                            }
                        }
                    }
                    _ => return false,
                }
            }
            // A raw control byte is not allowed in a JSON string; anything else is content.
            Some(c) if *c < 0x20 => return false,
            Some(_) => *at += 1,
        }
    }
}

/// `uXXXX` at the cursor (the backslash is already consumed): its value, cursor past it.
fn hex4(b: &[u8], at: &mut usize) -> Option<u32> {
    if b.get(*at) != Some(&b'u') {
        return None;
    }
    let digits = b.get(*at + 1..*at + 5)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let value = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
    *at += 5;
    Some(value)
}

fn skip_ws(b: &[u8], at: &mut usize) {
    while matches!(b.get(*at), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        *at += 1;
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// The three classes jq reads and serde_json refuses have to read as JSON here, or the hook
    /// would take them for a payload nobody can read and pass them in silence.
    #[test]
    fn r07__what_jq_reads_and_serde_json_refuses_is_json() {
        let deep = format!("{}1{}", "[".repeat(129), "]".repeat(129));
        for text in [
            r#"{"n":1e400}"#,
            r#"{"n":-1e400}"#,
            &format!(r#"{{"n":{}}}"#, "9".repeat(400)),
            r#"{"s":"\udc00"}"#,
            &deep,
        ] {
            assert!(is_json(text), "{text}");
            assert!(
                serde_json::from_str::<serde_json::Value>(text).is_err(),
                "{text}"
            );
        }
        // An exponent that underflows is read by both, with different values (serde_json 0.0, jq
        // the literal 1E-400): a divergence of what a number prints as, not of what parses.
        assert!(is_json(r#"{"n":1e-400}"#));
        assert!(serde_json::from_str::<serde_json::Value>(r#"{"n":1e-400}"#).is_ok());
    }

    #[test]
    fn r07__what_neither_reads_is_not_json() {
        let too_deep = format!("{}1{}", "[".repeat(257), "]".repeat(257));
        for text in [
            "this is not a JSON payload at all\n",
            "{",
            r#"{"a":}"#,
            r#"{"a" 1}"#,
            r#"{"s":"\ud800"}"#,
            r#"{"n":01}"#,
            r#"{"n":1e}"#,
            "{} {}",
            r#"{"s":"raw
newline"}"#,
            &too_deep,
        ] {
            assert!(!is_json(text), "{text}");
        }
    }

    #[test]
    fn r07__ordinary_payloads_read_as_json() {
        for text in [
            r#"{"cwd":".","tool_input":{"model":"fable","n":[1,2.5,-3e-2],"ok":true,"no":null}}"#,
            "[]",
            "{}",
            "  \"one\"  ",
            r#"{"s":"\ud83d\ude00 \u0041"}"#,
        ] {
            assert!(is_json(text), "{text}");
            assert!(
                serde_json::from_str::<serde_json::Value>(text).is_ok(),
                "{text}"
            );
        }
    }
}
