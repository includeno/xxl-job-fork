use serde::Serialize;

pub fn format_executor_request_curl(url: &str, token: Option<&str>, body: &str) -> String {
    let mut segments = Vec::new();
    segments.push(format!("curl -sS -X POST \"{}\"", url));
    segments.push("-H \"Content-Type: application/json\"".to_string());

    if let Some(value) = token.and_then(non_empty) {
        segments.push(format!("-H \"XXL-JOB-ACCESS-TOKEN: {}\"", value));
    }

    segments.push(format!("-d '{}'", escape_single_quotes(body)));

    let mut command = String::new();
    for (index, segment) in segments.iter().enumerate() {
        if index == 0 {
            command.push_str(segment);
        } else {
            command.push_str(" \\\n  ");
            command.push_str(segment);
        }
    }

    command
}

pub fn to_pretty_json<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_string_pretty(value).ok()
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn escape_single_quotes(input: &str) -> String {
    input.replace('\'', "'\"'\"'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_curl_without_token() {
        let command =
            format_executor_request_curl("http://localhost:9999/run", None, r#"{"key":"value"}"#);

        let expected = [
            r#"curl -sS -X POST "http://localhost:9999/run" \"#,
            r#"  -H "Content-Type: application/json" \"#,
            r#"  -d '{"key":"value"}'"#,
        ]
        .join("\n");

        assert_eq!(command, expected);
    }

    #[test]
    fn formats_curl_with_token_and_quotes() {
        let command = format_executor_request_curl(
            "http://localhost:9999/run",
            Some(" default_token "),
            r#"{"handler":"demo"}"#,
        );

        let expected = [
            r#"curl -sS -X POST "http://localhost:9999/run" \"#,
            r#"  -H "Content-Type: application/json" \"#,
            r#"  -H "XXL-JOB-ACCESS-TOKEN: default_token" \"#,
            r#"  -d '{"handler":"demo"}'"#,
        ]
        .join("\n");

        assert_eq!(command, expected);
    }
}
