// SPDX-License-Identifier: MIT

//! What an endpoint promises, read from its code (FEAT-179): the rules its
//! body is held to, and what it answers.
//!
//! The Java and NestJS readers find the facts — a `@Size(max = 50)`, a
//! `throw new NotFoundException()`, an `@ApiResponse(responseCode = "409")`
//! — and this module says them the same way for both: a status's words,
//! the error body each framework sends by default, and an exception's name
//! as a sentence. Answers are kept one per status, the most certain kept
//! when two say the same status: an annotation the author wrote over a
//! thrown exception, a thrown exception over a guess.

use super::super::model::{Answer, FieldRule};
use super::json::J;

/// How sure a source is: higher wins when two give the same status.
pub const DECLARED: u8 = 3;
pub const THROWN: u8 = 2;
pub const INFERRED: u8 = 1;

/// The words HTTP gives a status.
pub fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No content",
        400 => "Bad request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not found",
        405 => "Method not allowed",
        409 => "Conflict",
        410 => "Gone",
        413 => "Payload too large",
        415 => "Unsupported media type",
        422 => "Unprocessable entity",
        429 => "Too many requests",
        500 => "Internal server error",
        501 => "Not implemented",
        502 => "Bad gateway",
        503 => "Service unavailable",
        504 => "Gateway timeout",
        _ => "",
    }
}

/// `HttpStatus.NOT_FOUND`, `NOT_FOUND`, `Response.Status.CREATED`, `404`
/// or `HttpStatus.NOT_FOUND.value()` as its number.
pub fn status_of(text: &str) -> Option<u16> {
    let text = text.trim().trim_end_matches(".value()");
    if let Ok(number) = text.trim_matches('"').parse::<u16>() {
        return (100..600).contains(&number).then_some(number);
    }
    let name = text.rsplit('.').next().unwrap_or(text).to_ascii_uppercase();
    Some(match name.as_str() {
        "OK" => 200,
        "CREATED" => 201,
        "ACCEPTED" => 202,
        "NO_CONTENT" => 204,
        "BAD_REQUEST" => 400,
        "UNAUTHORIZED" => 401,
        "FORBIDDEN" => 403,
        "NOT_FOUND" => 404,
        "METHOD_NOT_ALLOWED" => 405,
        "CONFLICT" => 409,
        "GONE" => 410,
        "PAYLOAD_TOO_LARGE" => 413,
        "UNSUPPORTED_MEDIA_TYPE" => 415,
        "UNPROCESSABLE_ENTITY" | "UNPROCESSABLE_CONTENT" => 422,
        "TOO_MANY_REQUESTS" => 429,
        "INTERNAL_SERVER_ERROR" => 500,
        "NOT_IMPLEMENTED" => 501,
        "BAD_GATEWAY" => 502,
        "SERVICE_UNAVAILABLE" => 503,
        "GATEWAY_TIMEOUT" => 504,
        _ => return None,
    })
}

/// The status an exception's name gives it when nothing else does:
/// `OrderNotFoundException` is a 404, `DuplicateEmailException` a 409.
pub fn status_by_name(name: &str) -> Option<u16> {
    let lower = name.to_ascii_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| lower.contains(word));
    if has(&["notfound", "nosuch", "missing"]) {
        Some(404)
    } else if has(&["conflict", "alreadyexist", "duplicate", "exists"]) {
        Some(409)
    } else if has(&["unauthorized", "unauthenticated", "authentication"]) {
        Some(401)
    } else if has(&["forbidden", "accessdenied", "permission", "notallowed"]) {
        Some(403)
    } else if has(&["badrequest", "invalid", "validation", "illegalargument"]) {
        Some(400)
    } else if has(&["unprocessable"]) {
        Some(422)
    } else if has(&["toomany", "ratelimit"]) {
        Some(429)
    } else {
        None
    }
}

/// `OrderNotFoundException` as `Order not found`.
pub fn sentence(exception: &str) -> String {
    let name = exception
        .trim_end_matches("Exception")
        .trim_end_matches("Error")
        .trim_end_matches("Ex");
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (at, c) in chars.iter().enumerate() {
        let next_lower = chars.get(at + 1).is_some_and(|n| n.is_lowercase());
        let prev_lower = at > 0 && chars[at - 1].is_lowercase();
        if c.is_uppercase() && !word.is_empty() && (prev_lower || next_lower) {
            words.push(std::mem::take(&mut word));
        }
        word.push(*c);
    }
    if !word.is_empty() {
        words.push(word);
    }
    let text = words
        .iter()
        .map(|w| {
            if w.chars().all(|c| c.is_uppercase()) && w.len() > 1 {
                w.clone()
            } else {
                w.to_lowercase()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = text.chars();
    match out.next() {
        Some(first) => first.to_uppercase().collect::<String>() + out.as_str(),
        None => exception.to_string(),
    }
}

/// The body Spring Boot sends for an error it was not told how to say.
pub fn spring_error(status: u16, path: &str) -> String {
    J::Obj(vec![
        (
            "timestamp".into(),
            J::Str("2026-01-31T09:30:00.000+00:00".into()),
        ),
        ("status".into(), J::Num(status.to_string())),
        ("error".into(), J::Str(title(status))),
        ("path".into(), J::Str(path.to_string())),
    ])
    .pretty()
}

/// The body NestJS sends for an `HttpException`: the message, its status and
/// its words. `messages` is a list for a validation failure.
pub fn nest_error(status: u16, message: Option<&str>, messages: &[String]) -> String {
    let message = if messages.is_empty() {
        J::Str(message.map(str::to_string).unwrap_or_else(|| title(status)))
    } else {
        J::Arr(messages.iter().cloned().map(J::Str).collect())
    };
    let mut entries = vec![("message".to_string(), message)];
    if messages.is_empty() && message_is_title(status, &entries) {
        entries.clear();
        entries.push(("statusCode".into(), J::Num(status.to_string())));
        entries.push(("message".into(), J::Str(title(status))));
        return J::Obj(entries).pretty();
    }
    entries.push(("error".into(), J::Str(title(status))));
    entries.push(("statusCode".into(), J::Num(status.to_string())));
    J::Obj(entries).pretty()
}

fn message_is_title(status: u16, entries: &[(String, J)]) -> bool {
    matches!(&entries[0].1, J::Str(text) if *text == title(status))
}

/// A status's words as Title Case, as frameworks put them in a body.
fn title(status: u16) -> String {
    reason(status)
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The messages class-validator gives for the rules, as NestJS's
/// `ValidationPipe` returns them in a 400.
pub fn validator_messages(rules: &[FieldRule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        let name = rule
            .path
            .rsplit('.')
            .next()
            .unwrap_or(&rule.path)
            .trim_end_matches("[]");
        if rule.required {
            out.push(format!("{name} should not be empty"));
        }
        match rule.format.as_deref() {
            Some("email") => out.push(format!("{name} must be an email")),
            Some("uuid") => out.push(format!("{name} must be a UUID")),
            Some("date-time") => out.push(format!("{name} must be a valid ISO 8601 date string")),
            Some("uri") => out.push(format!("{name} must be a URL address")),
            _ => {}
        }
        if let Some(min) = rule.min_length {
            out.push(format!(
                "{name} must be longer than or equal to {min} characters"
            ));
        }
        if let Some(max) = rule.max_length {
            out.push(format!(
                "{name} must be shorter than or equal to {max} characters"
            ));
        }
        if let Some(min) = rule.minimum {
            out.push(format!("{name} must not be less than {}", number(min)));
        }
        if let Some(max) = rule.maximum {
            out.push(format!("{name} must not be greater than {}", number(max)));
        }
        if !rule.values.is_empty() {
            out.push(format!(
                "{name} must be one of the following values: {}",
                rule.values.join(", ")
            ));
        }
        if rule.pattern.is_some() {
            out.push(format!("{name} must match the expected pattern"));
        }
    }
    out.truncate(6);
    out
}

/// `3.0` as `3`, `0.5` as `0.5`.
pub fn number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

/// Answers kept one per status, the most certain of each, in status order.
#[derive(Default)]
pub struct Answers {
    kept: Vec<(u8, Answer)>,
}

impl Answers {
    pub fn add(&mut self, sure: u8, answer: Answer) {
        match self
            .kept
            .iter_mut()
            .find(|(_, kept)| kept.status == answer.status)
        {
            Some((was, kept)) if sure > *was => {
                // Keep an example the surer one does not have.
                let body = answer.body.clone().or_else(|| kept.body.take());
                *kept = Answer { body, ..answer };
                *was = sure;
            }
            Some((_, kept)) => {
                if kept.body.is_none() {
                    kept.body = answer.body;
                }
            }
            None => self.kept.push((sure, answer)),
        }
    }

    pub fn has(&self, status: u16) -> bool {
        self.kept.iter().any(|(_, kept)| kept.status == status)
    }

    pub fn done(mut self) -> Vec<Answer> {
        self.kept.sort_by_key(|(_, answer)| answer.status);
        self.kept.into_iter().map(|(_, answer)| answer).collect()
    }
}

/// An answer.
pub fn answer(
    status: u16,
    says: impl Into<String>,
    body: Option<String>,
    from: impl Into<String>,
) -> Answer {
    Answer {
        status,
        says: says.into(),
        body,
        from: from.into(),
    }
}

/// The success a verb answers by default, its words.
pub fn success_words(status: u16) -> &'static str {
    match status {
        201 => "Created",
        202 => "Accepted",
        204 => "Done, with no body",
        _ => "OK",
    }
}

/// The exceptions thrown in `body`: `throw new X(`, `-> new X(` in an
/// `orElseThrow`, and `throw X.of(`. With the first string each is given.
pub fn thrown(body: &str) -> Vec<(String, Option<String>)> {
    let pattern = regex::Regex::new(
        r#"(?:throw\s+new|->\s*new|=>\s*new|throw)\s+([A-Z][A-Za-z0-9_]*(?:Exception|Error))\s*\(\s*(?:"([^"]*)"|'([^']*)'|`([^`$]*)`)?"#,
    )
    .expect("a valid pattern");
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for found in pattern.captures_iter(body) {
        let name = found[1].to_string();
        let message = found
            .get(2)
            .or_else(|| found.get(3))
            .or_else(|| found.get(4))
            .map(|m| m.as_str().to_string())
            .filter(|m| !m.is_empty());
        if !out.iter().any(|(seen, _)| *seen == name) {
            out.push((name, message));
        }
    }
    out
}

/// The methods `body` calls on a field: `orderService.find(` and
/// `this.orders.find(` as `(orderService, find)`.
pub fn calls_on_fields(body: &str) -> Vec<(String, String)> {
    let pattern = regex::Regex::new(r"(?:this\.)?([a-z][A-Za-z0-9_]*)\.([a-z][A-Za-z0-9_]*)\s*\(")
        .expect("a valid pattern");
    let mut out: Vec<(String, String)> = Vec::new();
    for found in pattern.captures_iter(body) {
        let pair = (found[1].to_string(), found[2].to_string());
        if !out.contains(&pair) {
            out.push(pair);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_is_read_as_spring_jaxrs_or_nest_write_it() {
        assert_eq!(status_of("HttpStatus.NOT_FOUND"), Some(404));
        assert_eq!(status_of("Response.Status.CREATED"), Some(201));
        assert_eq!(status_of("HttpStatus.CREATED.value()"), Some(201));
        assert_eq!(status_of("204"), Some(204));
        assert_eq!(status_of("HttpStatus.I_AM_A_TEAPOT"), None);
    }

    #[test]
    fn an_exception_is_said_as_a_sentence_and_its_name_guesses_its_status() {
        assert_eq!(sentence("OrderNotFoundException"), "Order not found");
        assert_eq!(sentence("DuplicateEmailException"), "Duplicate email");
        assert_eq!(sentence("HTTPTimeoutError"), "HTTP timeout");
        assert_eq!(status_by_name("OrderNotFoundException"), Some(404));
        assert_eq!(status_by_name("EmailAlreadyExistsException"), Some(409));
        assert_eq!(status_by_name("PaymentDeclinedException"), None);
    }

    #[test]
    fn what_is_thrown_is_found_with_its_message() {
        let body = r#"
            Order order = repo.findById(id).orElseThrow(() -> new OrderNotFoundException("No order " + id));
            if (x) throw new IllegalStateException();
            throw new NotFoundException(`Order ${id} not found`);
            throw new ConflictException('Already paid');
        "#;
        assert_eq!(
            thrown(body),
            [
                (
                    "OrderNotFoundException".to_string(),
                    Some("No order ".to_string())
                ),
                ("IllegalStateException".to_string(), None),
                ("NotFoundException".to_string(), None),
                (
                    "ConflictException".to_string(),
                    Some("Already paid".to_string())
                )
            ]
        );
    }

    #[test]
    fn the_surer_answer_for_a_status_wins_and_keeps_an_example() {
        let mut answers = Answers::default();
        answers.add(
            THROWN,
            answer(
                404,
                "Order not found",
                Some("{}".into()),
                "throws OrderNotFoundException",
            ),
        );
        answers.add(DECLARED, answer(404, "No such order", None, "@ApiResponse"));
        answers.add(INFERRED, answer(200, "OK", None, "the return type"));
        answers.add(INFERRED, answer(404, "Not found", None, "a guess"));
        let done = answers.done();
        assert_eq!(done.len(), 2);
        assert_eq!(done[0].status, 200);
        assert_eq!(
            (done[1].says.as_str(), done[1].body.as_deref()),
            ("No such order", Some("{}"))
        );
    }

    #[test]
    fn nest_says_a_validation_failure_as_its_pipe_does() {
        let rules = vec![FieldRule {
            path: "email".into(),
            kind: "string".into(),
            required: true,
            format: Some("email".into()),
            ..FieldRule::default()
        }];
        let messages = validator_messages(&rules);
        assert_eq!(
            messages,
            ["email should not be empty", "email must be an email"]
        );
        let body = nest_error(400, None, &messages);
        assert!(body.contains("\"error\": \"Bad Request\""), "{body}");
        assert!(body.contains("\"statusCode\": 400"), "{body}");
    }
}
