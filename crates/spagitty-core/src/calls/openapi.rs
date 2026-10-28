// SPDX-License-Identifier: MIT

//! A collection as an OpenAPI 3.1 document (FEAT-179).
//!
//! The standard every other tool reads: Postman, Insomnia, Bruno, Swagger
//! UI, Stoplight and API gateways import it. Each REST call is an
//! operation, at its path with `{{id}}` and `:id` as `{id}`, tagged with
//! the folders it is in. Its query and headers are parameters with their
//! values as examples; its body is the request body, with a schema made
//! from the rules the code holds each field to (FEAT-179) — `minLength`,
//! `format: email`, `enum` — or, without them, from the example itself.
//! What it answers is its responses, an example each. SOAP calls are WSDL's
//! to describe, and are left out.

use serde_json::{json, Map, Value};
use yaml_rust2::{Yaml, YamlEmitter};

use super::import::json::J;
use super::import::path_of;
use super::model::{Body, BodyType, Call, Collection, FieldRule, Item, Kind};

/// The document for `collection`. `server` is where it is served, from an
/// environment's `baseUrl`; `/` without one. `product` says what wrote it.
pub fn document(collection: &Collection, server: Option<&str>, product: &str) -> Value {
    let mut paths = Map::new();
    let mut operation_ids: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    for (folders, call) in rest_calls(&collection.items, &[]) {
        let (path, path_params) = template(&call.url);
        let method = call.method().to_ascii_lowercase();
        let entry = paths
            .entry(path.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(operations) = entry.as_object_mut() else {
            continue;
        };
        if operations.contains_key(&method) {
            // Two calls to one endpoint: the first describes it.
            continue;
        }
        let tag = folders.join(" / ");
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag.clone());
        }
        operations.insert(
            method,
            operation(call, &tag, &path_params, &mut operation_ids),
        );
    }
    let servers = json!([{ "url": server.filter(|s| !s.trim().is_empty()).unwrap_or("/") }]);
    let mut out = json!({
        "openapi": "3.1.0",
        "info": {
            "title": collection.name,
            "version": "1.0.0",
            "description": format!("Exported from {product}.")
        },
        "servers": servers,
        "paths": paths
    });
    if !tags.is_empty() {
        out["tags"] = Value::Array(
            tags.into_iter()
                .map(|name| json!({ "name": name }))
                .collect(),
        );
    }
    out
}

/// Every REST call, with the folders it is in.
fn rest_calls<'a>(items: &'a [Item], folders: &[String]) -> Vec<(Vec<String>, &'a Call)> {
    let mut out = Vec::new();
    for item in items {
        match item {
            Item::Call(call) if call.kind == Kind::Rest => {
                out.push((folders.to_vec(), call.as_ref()))
            }
            Item::Folder(folder) => {
                let mut inner = folders.to_vec();
                inner.push(folder.folder.clone());
                out.extend(rest_calls(&folder.items, &inner));
            }
            _ => {}
        }
    }
    out
}

/// A call's URL as an OpenAPI path, and its path parameters' names:
/// `{{baseUrl}}/orders/{{id}}/lines` is `/orders/{id}/lines`.
fn template(url: &str) -> (String, Vec<String>) {
    let path = path_of(url);
    let path = path.split(['?', '#']).next().unwrap_or("");
    let mut names = Vec::new();
    let segments: Vec<String> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let name = segment
                .strip_prefix("{{")
                .and_then(|s| s.strip_suffix("}}"))
                .or_else(|| segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')))
                .or_else(|| segment.strip_prefix(':'));
            match name {
                Some(name) => {
                    let name = name.trim().to_string();
                    names.push(name.clone());
                    format!("{{{name}}}")
                }
                None => segment.to_string(),
            }
        })
        .collect();
    (format!("/{}", segments.join("/")), names)
}

/// A name for the operation, unique in the document: `createOrder`.
fn operation_id(name: &str, taken: &mut Vec<String>) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .enumerate()
        .map(|(at, word)| {
            let lower = word.to_lowercase();
            if at == 0 {
                lower
            } else {
                let mut chars = lower.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            }
        })
        .collect();
    let base = if words.is_empty() {
        "call".to_string()
    } else {
        words.concat()
    };
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}{n}");
        n += 1;
    }
    taken.push(id.clone());
    id
}

fn operation(call: &Call, tag: &str, path_params: &[String], taken: &mut Vec<String>) -> Value {
    let mut parameters: Vec<Value> = path_params
        .iter()
        .map(|name| json!({ "name": name, "in": "path", "required": true, "schema": { "type": "string" } }))
        .collect();
    if let Some((_, query)) = call.url.split_once('?') {
        for part in query.split('&').filter(|p| !p.is_empty()) {
            let (name, value) = part.split_once('=').unwrap_or((part, ""));
            let mut param = json!({ "name": name, "in": "query", "schema": { "type": "string" } });
            if !value.is_empty() && !value.starts_with("{{") {
                param["example"] = Value::String(value.to_string());
            }
            parameters.push(param);
        }
    }
    for header in call.headers.iter().filter(|h| h.on) {
        let lower = header.name.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "content-type" | "accept" | "authorization" | "content-length"
        ) {
            continue;
        }
        let mut param =
            json!({ "name": header.name, "in": "header", "schema": { "type": "string" } });
        if !header.value.is_empty() && !header.value.starts_with("{{") {
            param["example"] = Value::String(header.value.clone());
        }
        parameters.push(param);
    }

    let mut op = json!({
        "summary": call.name,
        "operationId": operation_id(&call.name, taken),
    });
    if !tag.is_empty() {
        op["tags"] = json!([tag]);
    }
    if !parameters.is_empty() {
        op["parameters"] = Value::Array(parameters);
    }
    if let Some(body) = call.body.as_ref().filter(|b| has_content(b)) {
        op["requestBody"] = request_body(body, &call.rules);
    }
    let mut responses = Map::new();
    for answer in &call.responses {
        let mut response = json!({ "description": if answer.says.is_empty() { "—" } else { answer.says.as_str() } });
        if let Some(example) = &answer.body {
            let value =
                serde_json::from_str::<Value>(example).unwrap_or(Value::String(example.clone()));
            response["content"] = json!({ "application/json": { "example": value } });
        }
        responses.insert(answer.status.to_string(), response);
    }
    if responses.is_empty() {
        responses.insert("200".into(), json!({ "description": "OK" }));
    }
    op["responses"] = Value::Object(responses);
    if let Some(notes) = call.notes.as_ref().filter(|n| !n.trim().is_empty()) {
        op["description"] = Value::String(notes.clone());
    }
    op
}

fn has_content(body: &Body) -> bool {
    body.text.as_ref().is_some_and(|t| !t.trim().is_empty()) || !body.fields.is_empty()
}

fn request_body(body: &Body, rules: &[FieldRule]) -> Value {
    let media = match body.kind {
        BodyType::Json => "application/json",
        BodyType::Form => "application/x-www-form-urlencoded",
        BodyType::Multipart => "multipart/form-data",
        BodyType::Xml => "application/xml",
        _ => "text/plain",
    };
    let example = match body.kind {
        BodyType::Form | BodyType::Multipart => Value::Object(
            body.fields
                .iter()
                .map(|f| (f.name.clone(), Value::String(f.value.clone())))
                .collect(),
        ),
        _ => {
            let text = body.text.clone().unwrap_or_default();
            serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text))
        }
    };
    let schema = if rules.is_empty() {
        inferred(&example)
    } else {
        from_rules(rules)
    };
    json!({ "required": true, "content": { media: { "schema": schema, "example": example } } })
}

/// A schema from the example alone: its shape, without bounds.
fn inferred(value: &Value) -> Value {
    match value {
        Value::Object(entries) => json!({
            "type": "object",
            "properties": entries.iter().map(|(k, v)| (k.clone(), inferred(v))).collect::<Map<_, _>>()
        }),
        Value::Array(items) => {
            json!({ "type": "array", "items": items.first().map(inferred).unwrap_or(json!({})) })
        }
        Value::String(_) => json!({ "type": "string" }),
        Value::Number(n) if n.is_i64() || n.is_u64() => json!({ "type": "integer" }),
        Value::Number(_) => json!({ "type": "number" }),
        Value::Bool(_) => json!({ "type": "boolean" }),
        Value::Null => json!({}),
    }
}

/// A schema from the code's rules: each rule placed at its path, its bounds
/// as OpenAPI's keywords, and each required field in its parent's list.
pub fn from_rules(rules: &[FieldRule]) -> Value {
    let mut root = json!({ "type": "object", "properties": {} });
    for rule in rules {
        let mut segments: Vec<&str> = rule.path.split('.').collect();
        let Some(last) = segments.pop() else { continue };
        let mut at = &mut root;
        for segment in segments {
            at = child(at, segment);
        }
        let (name, many) = match last.strip_suffix("[]") {
            Some(name) => (name, true),
            None => (last, false),
        };
        if rule.required {
            let required = at
                .as_object_mut()
                .expect("an object schema")
                .entry("required")
                .or_insert(json!([]));
            if let Some(list) = required.as_array_mut() {
                if !list.iter().any(|v| v == name) {
                    list.push(Value::String(name.to_string()));
                }
            }
        }
        let properties = at
            .as_object_mut()
            .expect("an object schema")
            .entry("properties")
            .or_insert(json!({}));
        let slot = properties
            .as_object_mut()
            .expect("properties")
            .entry(name.to_string())
            .or_insert(json!({}));
        let target = if many {
            slot["type"] = json!("array");
            slot.as_object_mut()
                .expect("a schema")
                .entry("items")
                .or_insert(json!({}))
        } else {
            slot
        };
        apply(target, rule);
    }
    root
}

/// The schema at `segment` under `at`, made where missing: `lines[]` is the
/// items of the array `lines`.
fn child<'a>(at: &'a mut Value, segment: &str) -> &'a mut Value {
    let (name, many) = match segment.strip_suffix("[]") {
        Some(name) => (name, true),
        None => (segment, false),
    };
    let properties = at
        .as_object_mut()
        .expect("an object schema")
        .entry("properties")
        .or_insert(json!({}));
    let slot = properties
        .as_object_mut()
        .expect("properties")
        .entry(name.to_string())
        .or_insert(json!({}));
    if many {
        slot["type"] = json!("array");
        let items = slot
            .as_object_mut()
            .expect("a schema")
            .entry("items")
            .or_insert(json!({}));
        if items.get("type").is_none() {
            items["type"] = json!("object");
        }
        items
    } else {
        if slot.get("type").is_none() {
            slot["type"] = json!("object");
        }
        slot
    }
}

fn apply(schema: &mut Value, rule: &FieldRule) {
    let kind = if rule.kind == "array" && schema.get("type") == Some(&json!("array")) {
        // `lines` is an array whose items' rules come under `lines[]`.
        return;
    } else {
        rule.kind.as_str()
    };
    if schema.get("type").is_none() || kind != "object" {
        schema["type"] = json!(kind);
    }
    if let Some(format) = &rule.format {
        schema["format"] = json!(format);
    }
    if let Some(min) = rule.minimum {
        schema["minimum"] = number(min);
    }
    if let Some(max) = rule.maximum {
        schema["maximum"] = number(max);
    }
    if let Some(min) = rule.min_length {
        schema["minLength"] = json!(min);
    }
    if let Some(max) = rule.max_length {
        schema["maxLength"] = json!(max);
    }
    if let Some(pattern) = &rule.pattern {
        schema["pattern"] = json!(pattern);
    }
    if !rule.values.is_empty() {
        schema["enum"] = json!(rule.values);
    }
}

fn number(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// The order OpenAPI documents are written in, the way people read them:
/// `openapi` first, an operation's summary before its parameters. Keys not
/// named here — paths, statuses, property names — come after, sorted.
const ORDER: [&str; 31] = [
    "openapi",
    "info",
    "title",
    "version",
    "description",
    "servers",
    "url",
    "tags",
    "paths",
    "summary",
    "operationId",
    "parameters",
    "name",
    "in",
    "required",
    "schema",
    "requestBody",
    "content",
    "example",
    "responses",
    "type",
    "format",
    "enum",
    "minimum",
    "maximum",
    "minLength",
    "maxLength",
    "pattern",
    "items",
    "properties",
    "variables",
];

fn ordered(entries: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut out: Vec<(&String, &Value)> = entries.iter().collect();
    let rank = |key: &str| {
        ORDER
            .iter()
            .position(|known| *known == key)
            .unwrap_or(ORDER.len())
    };
    out.sort_by(|(a, _), (b, _)| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
    out
}

/// The document as JSON, two spaces, keys in OpenAPI's order.
pub fn json(value: &Value) -> String {
    to_j(value).pretty() + "\n"
}

fn to_j(value: &Value) -> J {
    match value {
        Value::Null => J::Null,
        Value::Bool(b) => J::Bool(*b),
        Value::Number(n) => J::Num(n.to_string()),
        Value::String(s) => J::Str(s.clone()),
        Value::Array(items) => J::Arr(items.iter().map(to_j).collect()),
        Value::Object(entries) => J::Obj(
            ordered(entries)
                .into_iter()
                .map(|(k, v)| (k.clone(), to_j(v)))
                .collect(),
        ),
    }
}

/// The document as YAML, keys in OpenAPI's order.
pub fn yaml(value: &Value) -> String {
    let mut out = String::new();
    let mut emitter = YamlEmitter::new(&mut out);
    if emitter.dump(&to_yaml(value)).is_err() {
        return serde_json::to_string_pretty(value).unwrap_or_default();
    }
    // The emitter starts a document with `---`; a file does not need it.
    out.strip_prefix("---\n").map(str::to_string).unwrap_or(out) + "\n"
}

fn to_yaml(value: &Value) -> Yaml {
    match value {
        Value::Null => Yaml::Null,
        Value::Bool(b) => Yaml::Boolean(*b),
        Value::Number(n) if n.is_i64() => Yaml::Integer(n.as_i64().unwrap_or_default()),
        Value::Number(n) => Yaml::Real(n.to_string()),
        Value::String(s) => Yaml::String(s.clone()),
        Value::Array(items) => Yaml::Array(items.iter().map(to_yaml).collect()),
        Value::Object(entries) => {
            let mut hash = yaml_rust2::yaml::Hash::new();
            for (key, value) in ordered(entries) {
                hash.insert(Yaml::String(key.clone()), to_yaml(value));
            }
            Yaml::Hash(hash)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calls::model::{Answer, Folder, Pair};

    fn collection() -> Collection {
        let create = Call {
            id: "c1".into(),
            name: "Open account".into(),
            method: Some("POST".into()),
            url: "{{baseUrl}}/accounts?dryRun=false".into(),
            headers: vec![
                Pair::new("X-Tenant", "acme"),
                Pair::new("Content-Type", "application/json"),
            ],
            body: Some(Body {
                kind: BodyType::Json,
                text: Some(
                    r#"{"email":"mira@example.com","age":18,"addresses":[{"line":"1 Main St"}]}"#
                        .into(),
                ),
                ..Body::default()
            }),
            rules: vec![
                FieldRule {
                    path: "email".into(),
                    kind: "string".into(),
                    required: true,
                    format: Some("email".into()),
                    ..FieldRule::default()
                },
                FieldRule {
                    path: "age".into(),
                    kind: "integer".into(),
                    minimum: Some(18.0),
                    maximum: Some(130.0),
                    ..FieldRule::default()
                },
                FieldRule {
                    path: "addresses".into(),
                    kind: "array".into(),
                    ..FieldRule::default()
                },
                FieldRule {
                    path: "addresses[].line".into(),
                    kind: "string".into(),
                    required: true,
                    min_length: Some(1),
                    ..FieldRule::default()
                },
            ],
            responses: vec![
                Answer {
                    status: 201,
                    says: "Created".into(),
                    body: Some(r#"{"id":"a1"}"#.into()),
                    from: "the return type".into(),
                },
                Answer {
                    status: 409,
                    says: "That email has an account already".into(),
                    body: None,
                    from: "@ApiResponse".into(),
                },
            ],
            ..Call::default()
        };
        let get = Call {
            id: "c2".into(),
            name: "Get account".into(),
            url: "{{baseUrl}}/accounts/{{id}}".into(),
            ..Call::default()
        };
        Collection {
            v: 1,
            name: "accounts".into(),
            wsdl: None,
            vars: Vec::new(),
            items: vec![Item::Folder(Folder {
                folder: "accounts-service".into(),
                items: vec![Item::Folder(Folder {
                    folder: "AccountController".into(),
                    items: vec![Item::Call(Box::new(create)), Item::Call(Box::new(get))],
                })],
            })],
        }
    }

    #[test]
    fn a_collection_is_an_openapi_document_with_its_rules_and_answers() {
        let doc = document(&collection(), Some("http://localhost:8080"), "Workbench");
        assert_eq!(doc["openapi"], "3.1.0");
        assert_eq!(doc["servers"][0]["url"], "http://localhost:8080");
        assert_eq!(
            doc["tags"][0]["name"],
            "accounts-service / AccountController"
        );

        let open = &doc["paths"]["/accounts"]["post"];
        assert_eq!(open["operationId"], "openAccount");
        let names: Vec<&str> = open["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["dryRun", "X-Tenant"]);
        let schema = &open["requestBody"]["content"]["application/json"]["schema"];
        assert_eq!(schema["required"], json!(["email"]));
        assert_eq!(
            schema["properties"]["email"],
            json!({ "type": "string", "format": "email" })
        );
        assert_eq!(schema["properties"]["age"]["minimum"], 18);
        assert_eq!(
            schema["properties"]["addresses"]["items"]["required"],
            json!(["line"])
        );
        assert_eq!(
            schema["properties"]["addresses"]["items"]["properties"]["line"]["minLength"],
            1
        );
        assert_eq!(
            open["requestBody"]["content"]["application/json"]["example"]["age"],
            18
        );
        assert_eq!(
            open["responses"]["201"]["content"]["application/json"]["example"]["id"],
            "a1"
        );
        assert_eq!(
            open["responses"]["409"]["description"],
            "That email has an account already"
        );

        let get = &doc["paths"]["/accounts/{id}"]["get"];
        assert_eq!(
            get["parameters"][0],
            json!({ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } })
        );
        assert_eq!(get["responses"]["200"]["description"], "OK");
    }

    #[test]
    fn the_document_reads_as_yaml_too() {
        let text = yaml(&document(&collection(), None, "Workbench"));
        assert!(text.starts_with("openapi: 3.1.0\n"), "{text}");
        // A path with a variable is quoted: braces start a flow mapping in YAML.
        assert!(text.contains("  \"/accounts/{id}\":"), "{text}");
        assert!(text.contains("url: /"), "{text}");
    }
}
