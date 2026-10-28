// SPDX-License-Identifier: MIT

//! An agent's read of a controller (FEAT-179): what the scan could not tell
//! from the code's annotations alone.
//!
//! The scan reads rules and answers from what the code declares. An agent
//! the repository allows can be asked, from Import, to read the controller
//! as a person would — a status set in a handler's `if`, a message built in
//! a helper — and say the same things in the same shape. What it says is
//! added to what the code said and never replaces it: a field or a status
//! the scan already has stays as the scan read it. Each answer it adds says
//! it came from the agent.

use serde::Deserialize;
use serde_json::Value;

use super::super::model::{Answer, FieldRule};
use super::{contract, normalise, Endpoint};

/// The most of one file an agent is given.
pub const MAX_FILE: usize = 96 * 1024;

/// What the agent is told it is.
pub const SYSTEM: &str = "You read one source file and describe its HTTP endpoints as JSON. You run no command and edit no file.";

/// The question for one file and the endpoints the scan found in it, with
/// what the scan already knows of each.
pub fn prompt(file: &str, text: &str, endpoints: &[&Endpoint]) -> String {
    let mut listed = String::new();
    for endpoint in endpoints {
        listed.push_str(&format!("- {} {}", endpoint.method, endpoint.path));
        if let Some(line) = endpoint.line {
            listed.push_str(&format!(" (line {line})"));
        }
        let statuses: Vec<String> = endpoint
            .call
            .responses
            .iter()
            .map(|a| a.status.to_string())
            .collect();
        if !statuses.is_empty() {
            listed.push_str(&format!("; already known: answers {}", statuses.join(", ")));
        }
        if !endpoint.call.rules.is_empty() {
            listed.push_str(&format!(
                "; {} body fields known",
                endpoint.call.rules.len()
            ));
        }
        listed.push('\n');
    }
    let text = cut(text, MAX_FILE);
    format!(
        "This is {file}. It declares these endpoints:\n{listed}\n\
         For each one, say what its request body's fields must be and what it \
         answers with each HTTP status, as the code behaves: validation, checks \
         in the handler, exceptions it or what it calls throws, and how they \
         are turned into responses. Say only what the code shows; leave out \
         what you would be guessing.\n\n\
         Answer with JSON alone, no prose and no code fence, in this shape:\n\
         {{\"endpoints\":[{{\"method\":\"POST\",\"path\":\"/orders\",\
         \"rules\":[{{\"path\":\"customer.email\",\"type\":\"string\",\"required\":true,\
         \"format\":\"email\",\"minLength\":3,\"maxLength\":50,\"minimum\":1,\"maximum\":99,\
         \"pattern\":\"^[A-Z]+$\",\"values\":[\"OPEN\",\"SHUT\"]}}],\
         \"responses\":[{{\"status\":404,\"says\":\"Order not found\",\
         \"body\":{{\"message\":\"Order not found\"}}}}]}}]}}\n\
         A field inside an array is `lines[].qty`. Leave out a key you have no \
         value for.\n\n```\n{text}\n```"
    )
}

/// `text` cut to at most `max` bytes, on a character's edge.
fn cut(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// One endpoint as the agent described it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Read {
    pub method: String,
    pub path: String,
    pub rules: Vec<FieldRule>,
    pub responses: Vec<Answer>,
}

#[derive(Deserialize)]
struct Said {
    #[serde(default)]
    endpoints: Vec<SaidEndpoint>,
}

#[derive(Deserialize)]
struct SaidEndpoint {
    #[serde(default)]
    method: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    rules: Vec<Value>,
    #[serde(default)]
    responses: Vec<Value>,
}

/// What the agent said, read. Prose around the JSON and a code fence are
/// tolerated; a rule or an answer that is not in the shape asked is left out.
pub fn read(said: &str) -> Result<Vec<Read>, String> {
    let start = said.find('{');
    let end = said.rfind('}');
    let (Some(start), Some(end)) = (start, end) else {
        return Err("The agent did not answer with JSON.".into());
    };
    if end < start {
        return Err("The agent did not answer with JSON.".into());
    }
    let parsed: Said = serde_json::from_str(&said[start..=end])
        .map_err(|e| format!("The agent's answer could not be read: {e}"))?;
    Ok(parsed
        .endpoints
        .into_iter()
        .map(|endpoint| Read {
            method: endpoint.method.trim().to_ascii_uppercase(),
            path: endpoint.path.trim().to_string(),
            rules: endpoint
                .rules
                .into_iter()
                .filter_map(|rule| serde_json::from_value::<FieldRule>(rule).ok())
                .filter(|rule| !rule.path.trim().is_empty())
                .collect(),
            responses: endpoint
                .responses
                .into_iter()
                .filter_map(answer_of)
                .collect(),
        })
        .collect())
}

/// An answer as the agent wrote it: its body an object or a string.
fn answer_of(value: Value) -> Option<Answer> {
    let status = match value.get("status")? {
        Value::Number(n) => n.as_u64()?,
        Value::String(s) => s.trim().parse().ok()?,
        _ => return None,
    };
    let status = u16::try_from(status)
        .ok()
        .filter(|s| (100..600).contains(s))?;
    let says = value
        .get("says")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| contract::reason(status).to_string());
    let body = match value.get("body") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if text.trim().is_empty() => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(other) => serde_json::to_string_pretty(other).ok(),
    };
    Some(Answer {
        status,
        says,
        body,
        from: String::new(),
    })
}

/// Add what the agent read of `endpoint` to it, what the scan had first.
/// `agent` is its name, which each added answer says it is from. Answers
/// how many rules and answers were added.
pub fn add(endpoint: &mut Endpoint, reads: &[Read], agent: &str) -> usize {
    let key = normalise(&endpoint.path);
    let Some(read) = reads.iter().find(|read| {
        read.method.eq_ignore_ascii_case(&endpoint.method) && normalise(&read.path) == key
    }) else {
        return 0;
    };
    let mut rules = endpoint.call.rules.clone();
    let mut responses = endpoint.call.responses.clone();
    let mut added = 0;
    for rule in &read.rules {
        if !rules.iter().any(|kept| kept.path == rule.path) {
            rules.push(rule.clone());
            added += 1;
        }
    }
    for answer in &read.responses {
        if !responses.iter().any(|kept| kept.status == answer.status) {
            responses.push(Answer {
                from: agent.to_string(),
                ..answer.clone()
            });
            added += 1;
        }
    }
    if added == 0 {
        return 0;
    }
    responses.sort_by_key(|answer| answer.status);
    if let Some(declared) = endpoint.declared.as_mut() {
        declared.rules = rules.clone();
        declared.responses = responses.clone();
    }
    endpoint.call.rules = rules;
    endpoint.call.responses = responses;
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calls::import::{build, Choices, Declared, Status};

    fn endpoint(method: &str, path: &str) -> Endpoint {
        let declared = Declared {
            method: method.into(),
            path: path.into(),
            rules: vec![FieldRule {
                path: "email".into(),
                kind: "string".into(),
                required: true,
                ..Default::default()
            }],
            responses: vec![contract::answer(201, "Created", None, "the return type")],
            ..Default::default()
        };
        Endpoint {
            id: "1".into(),
            key: format!("{method} {}", normalise(path)),
            source: "code".into(),
            group: "OrderController".into(),
            file: Some("src/OrderController.java".into()),
            line: Some(12),
            method: method.into(),
            path: path.into(),
            hint: String::new(),
            call: build(&declared, "Create order", Choices::default()),
            status: Status::New,
            declared: Some(declared),
            folder: None,
        }
    }

    #[test]
    fn the_question_lists_what_the_scan_knows_and_cuts_a_long_file() {
        let found = endpoint("POST", "/orders/{id}");
        let asked = prompt(
            "src/OrderController.java",
            &"x".repeat(MAX_FILE + 10),
            &[&found],
        );
        assert!(asked.contains(
            "- POST /orders/{id} (line 12); already known: answers 201; 1 body fields known"
        ));
        assert!(asked.contains("JSON alone"));
        assert!(asked.len() < MAX_FILE + 4096);
    }

    #[test]
    fn an_answer_is_read_through_prose_and_a_fence() {
        let said = "Here it is:\n```json\n{\"endpoints\":[{\"method\":\"post\",\"path\":\"/orders/:id\",\
            \"rules\":[{\"path\":\"qty\",\"type\":\"integer\",\"minimum\":1},{\"type\":\"string\"}],\
            \"responses\":[{\"status\":\"409\",\"says\":\"Already paid\",\"body\":{\"message\":\"paid\"}},\
            {\"status\":422},{\"status\":999},{\"says\":\"no status\"}]}]}\n```";
        let reads = read(said).unwrap();
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].method, "POST");
        assert_eq!(reads[0].rules.len(), 1, "a rule without a path is left out");
        assert_eq!(reads[0].rules[0].minimum, Some(1.0));
        let statuses: Vec<u16> = reads[0].responses.iter().map(|a| a.status).collect();
        assert_eq!(statuses, vec![409, 422]);
        assert_eq!(reads[0].responses[1].says, "Unprocessable entity");
        assert!(reads[0].responses[0]
            .body
            .as_deref()
            .unwrap()
            .contains("\"message\": \"paid\""));
        assert!(read("I could not read it.").is_err());
    }

    #[test]
    fn what_the_agent_adds_never_replaces_what_the_scan_read() {
        let mut found = endpoint("POST", "/orders/{id}");
        let reads = vec![Read {
            method: "POST".into(),
            path: "/orders/{orderId}".into(),
            rules: vec![
                FieldRule {
                    path: "email".into(),
                    kind: "integer".into(),
                    ..Default::default()
                },
                FieldRule {
                    path: "qty".into(),
                    kind: "integer".into(),
                    minimum: Some(1.0),
                    ..Default::default()
                },
            ],
            responses: vec![
                contract::answer(201, "Made", None, ""),
                contract::answer(404, "Order not found", None, ""),
            ],
        }];
        assert_eq!(add(&mut found, &reads, "Claude Code"), 2);
        assert_eq!(found.call.rules[0].kind, "string", "the scan's rule stays");
        assert_eq!(found.call.rules[1].path, "qty");
        assert_eq!(found.call.responses[0].says, "Created");
        assert_eq!(found.call.responses[1].from, "Claude Code");
        let declared = found.declared.as_ref().unwrap();
        assert_eq!(
            declared.responses.len(),
            2,
            "bring-in builds from what was declared"
        );
        assert_eq!(declared.rules.len(), 2);

        assert_eq!(
            add(&mut found, &reads, "Claude Code"),
            0,
            "asked twice, nothing more"
        );
        let mut other = endpoint("GET", "/orders/{id}");
        assert_eq!(
            add(&mut other, &reads, "Claude Code"),
            0,
            "another method is another endpoint"
        );
    }
}
