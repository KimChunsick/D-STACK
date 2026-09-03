// verbs/hook/json.rs
// The hook payload as jq read it: a document that does not parse answers nothing, and the key
// order of an object survives, because the model rewrite hands tool_input back to Claude Code.

use std::fmt;

use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

use super::wellformed;

/// A JSON value that remembers the order its object keys arrived in. serde_json's own Value keeps
/// a sorted map, while `$ti + {model:"opus"}` gave the caller its keys back in the order it sent
/// them, so the port carries the order along.
#[derive(Clone)]
pub(super) enum Json {
    Object(Vec<(String, Json)>),
    Array(Vec<Json>),
    Text(String),
    /// A number, true, false or null, printed the way serde_json read it.
    Plain(String),
}

impl Json {
    /// The compact line `jq -c` prints: no spaces anywhere, non-ASCII left as UTF-8.
    pub(super) fn compact(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            Json::Text(text) => out.push_str(&quote(text)),
            Json::Plain(text) => out.push_str(text),
            Json::Array(items) => {
                out.push('[');
                for (at, item) in items.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Json::Object(pairs) => {
                out.push('{');
                for (at, (key, value)) in pairs.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    out.push_str(&quote(key));
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

/// A JSON string literal, escaped as serde_json escapes it — the escapes jq prints.
pub(super) fn quote(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| String::from("\"\""))
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Json, D::Error> {
        deserializer.deserialize_any(Any)
    }
}

struct Any;

impl<'de> Visitor<'de> for Any {
    type Value = Json;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Json, E> {
        Ok(Json::Plain(value.to_string()))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Json, E> {
        Ok(Json::Plain(value.to_string()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Json, E> {
        Ok(Json::Plain(value.to_string()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Json, E> {
        Ok(Json::Plain(
            serde_json::Number::from_f64(value)
                .map_or_else(|| "null".to_string(), |n| n.to_string()),
        ))
    }

    fn visit_str<E>(self, value: &str) -> Result<Json, E> {
        Ok(Json::Text(value.to_string()))
    }

    fn visit_unit<E>(self) -> Result<Json, E> {
        Ok(Json::Plain("null".to_string()))
    }

    fn visit_none<E>(self) -> Result<Json, E> {
        Ok(Json::Plain("null".to_string()))
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Json, D::Error> {
        deserializer.deserialize_any(Any)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Json, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Json::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Json, A::Error> {
        let mut pairs: Vec<(String, Json)> = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Json>()? {
            // A repeated member is collapsed onto the first one with the last value, which is what
            // jq answers on a lookup and what it prints back: {"a":"1","b":"2","a":"3"} reads as
            // {"a":"3","b":"2"}.
            match pairs.iter_mut().find(|(name, _)| *name == key) {
                Some(seen) => seen.1 = value,
                None => pairs.push((key, value)),
            }
        }
        Ok(Json::Object(pairs))
    }
}

/// What the hook was handed on stdin.
pub(super) struct Payload {
    root: Json,
}

impl Payload {
    /// What the hook can see of the payload, and None when it cannot see it at all.
    ///
    /// A payload jq could not read either leaves every field empty, which is what the shell's
    /// `jq ... 2>/dev/null || true` left behind, and every event then takes its "nothing to judge"
    /// branch. A payload jq reads and serde_json refuses is a different thing entirely: a number
    /// outside f64, nesting past serde_json's 128 (jq stops at 256) or a lone low surrogate would
    /// erase tool_name and let a model rewrite or a Korean check be skipped in silence, so it
    /// answers None and the caller blocks instead.
    pub(super) fn parse(text: &str) -> Option<Payload> {
        match serde_json::from_str(text) {
            Ok(root) => Some(Payload { root }),
            Err(_) if wellformed::is_json(text) => None,
            Err(_) => Some(Payload::empty()),
        }
    }

    /// The `{}` a payload nobody could read reads as.
    pub(super) fn empty() -> Payload {
        Payload {
            root: Json::Object(Vec::new()),
        }
    }

    /// `jq -r '<path> // empty'`: the text of a field, where null and false read as nothing and a
    /// path through a value that is not an object reads as nothing either.
    pub(super) fn field(&self, path: &str) -> String {
        let mut at = &self.root;
        for key in path.split('.') {
            at = match at {
                Json::Object(pairs) => match pairs.iter().find(|(name, _)| name == key) {
                    Some((_, value)) => value,
                    None => return String::new(),
                },
                _ => return String::new(),
            };
        }
        match at {
            Json::Text(text) => text.clone(),
            Json::Plain(text) if text == "null" || text == "false" => String::new(),
            other => other.compact(),
        }
    }

    /// `jq -c '.tool_input // {}'` and what `$ti + {model:"opus"}` then does with it: an object —
    /// or the `{}` that jq's `//` falls back to for null, false and a field that is not there — can
    /// be added to, and every other value makes jq fail. None is that failure, which the caller
    /// turns into a block: a payload the hook cannot rewrite must not pass as one it approved.
    pub(super) fn tool_input(&self) -> Option<&[(String, Json)]> {
        let Json::Object(pairs) = &self.root else {
            // A payload that is not an object at all: jq cannot index it, and the shell's
            // `|| printf '{}'` answers the empty object.
            return Some(&[]);
        };
        match pairs.iter().find(|(name, _)| name == "tool_input") {
            None => Some(&[]),
            Some((_, Json::Object(input))) => Some(input),
            Some((_, Json::Plain(text))) if text == "null" || text == "false" => Some(&[]),
            Some(_) => None,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// A payload both parsers read.
    fn read(text: &str) -> Payload {
        Payload::parse(text).expect("both parsers read this one")
    }

    /// `jq -c '.tool_input // {}'` answered `{}`: an object addition with no key to add.
    fn empty(payload: &Payload) -> bool {
        payload.tool_input().is_some_and(|input| input.is_empty())
    }

    #[test]
    fn r07__a_field_reads_as_jq_read_it() {
        let payload = read(
            r#"{"cwd":".","stop_hook_active":false,"n":3,"tool_input":{"model":"fable"},"t":null}"#,
        );
        assert_eq!(payload.field("cwd"), ".");
        // `// empty` swallows false and null; a number prints as its text.
        assert_eq!(payload.field("stop_hook_active"), "");
        assert_eq!(payload.field("t"), "");
        assert_eq!(payload.field("n"), "3");
        assert_eq!(payload.field("tool_input.model"), "fable");
        assert_eq!(payload.field("tool_input.nothing"), "");
        assert_eq!(payload.field("cwd.deeper"), "");
        assert_eq!(payload.field("tool_input"), r#"{"model":"fable"}"#);
    }

    #[test]
    fn r07__a_payload_that_does_not_parse_answers_nothing() {
        let payload = read("this is not a JSON payload at all\n");
        assert_eq!(payload.field("tool_name"), "");
        assert!(empty(&payload));
    }

    #[test]
    fn r07__object_keys_keep_the_order_they_arrived_in() {
        let payload =
            read(r#"{"tool_input":{"z":"last","a":"first","n":[1,{"b":true,"a":null}]}}"#);
        let input = Json::Object(payload.tool_input().expect("an object").to_vec());
        assert_eq!(
            input.compact(),
            r#"{"z":"last","a":"first","n":[1,{"b":true,"a":null}]}"#
        );
    }

    #[test]
    fn r07__a_tool_input_that_cannot_be_added_to_has_no_keys() {
        // jq's `// {}` catches exactly null and false; the rest reaches the object addition.
        for text in [
            r#"{"tool_input":null}"#,
            r#"{"tool_input":false}"#,
            "{}",
            "not json",
        ] {
            assert!(empty(&read(text)), "{text}");
        }
        for text in [
            r#"{"tool_input":[]}"#,
            r#"{"tool_input":"x"}"#,
            r#"{"tool_input":3}"#,
            r#"{"tool_input":true}"#,
        ] {
            assert!(read(text).tool_input().is_none(), "{text}");
        }
    }

    #[test]
    fn r07__a_repeated_member_reads_as_its_last_value() {
        let payload = read(r#"{"tool_input":{"a":"1","model":"sonnet","b":"2","model":"fable"}}"#);
        assert_eq!(payload.field("tool_input.model"), "fable");
        let input = Json::Object(payload.tool_input().expect("an object").to_vec());
        assert_eq!(input.compact(), r#"{"a":"1","model":"fable","b":"2"}"#);
    }

    /// The finding of round 063: a payload jq reads and serde_json refuses is not an empty one.
    #[test]
    fn r07__a_payload_only_jq_reads_is_no_payload_at_all() {
        for text in [
            r#"{"tool_name":"Agent","tool_input":{"model":"fable","n":1e400}}"#,
            r#"{"tool_name":"Agent","n":-1e400}"#,
            r#"{"tool_name":"Agent","s":"\udc00"}"#,
        ] {
            assert!(Payload::parse(text).is_none(), "{text}");
        }
        // Neither reads this one, and that keeps answering the way the reference answers.
        assert!(empty(&read("this is not a JSON payload at all\n")));
    }

    #[test]
    fn r07__strings_are_escaped_the_way_jq_escapes_them() {
        let text = Json::Text("line\none\t\"two\" — 정본".to_string());
        assert_eq!(text.compact(), "\"line\\none\\t\\\"two\\\" — 정본\"");
    }
}
