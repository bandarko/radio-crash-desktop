use serde_json::Value;
pub fn parse(v: &Value) -> Option<(String, String)> {
    let raw = v
        .get("currentsong")
        .or_else(|| v.get("songtitle"))?
        .as_str()?
        .trim();
    if raw.is_empty() {
        return None;
    }
    Some(match raw.split_once(" - ") {
        Some((a, t)) => (a.trim().into(), t.trim().into()),
        None => (String::new(), raw.into()),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn croatian() {
        assert_eq!(
            parse(&serde_json::json!({"songtitle":"Šarlo - Čuvaj me - uživo"})),
            Some(("Šarlo".into(), "Čuvaj me - uživo".into()))
        );
        assert_eq!(parse(&serde_json::json!({})), None);
    }
}
