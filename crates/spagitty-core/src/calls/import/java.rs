// SPDX-License-Identifier: MIT

//! Endpoints declared in Java (FEAT-162, proposal §8.2): Spring MVC and
//! WebFlux controllers, and JAX-RS resources.
//!
//! Parsed with the tree-sitter Java grammar Files already compiles in, and
//! read the way a person reads a controller: the class's mapping, each
//! method's mapping and its annotated parameters. A request body's example
//! is made from its class or record wherever it is in the scanned files,
//! field by field to a depth of four, with Bean Validation annotations
//! shaping the samples (`@Email` is an address, `@Min(1)` is 1).
//!
//! A constant used as a path (`@GetMapping(Paths.ORDERS)`) is looked up
//! among the `static final String`s of the scanned files; what cannot be
//! resolved is left as written, so the path still says where it came from.
//!
//! # What an endpoint promises (FEAT-179)
//!
//! The body's class gives the rules each field is held to, and the method
//! gives what it answers: its success from `@ResponseStatus` or the
//! `ResponseEntity` it builds, with the return type as the example; a 400
//! when the body is `@Valid`; a 401 and 403 under `@PreAuthorize`,
//! `@Secured` or `@RolesAllowed`; and each exception it throws — or the
//! service it calls throws, one call deep — at the status its
//! `@ResponseStatus`, a `@ControllerAdvice`'s `@ExceptionHandler`, or
//! failing those its name says. springdoc's `@ApiResponse` is the author's
//! own word and is taken over the rest.

use std::collections::HashMap;

use tree_sitter::{Node, Parser};

use super::super::model::{new_id, Answer, Call, FieldRule, Kind, WsMode, WsSecurity};
use super::super::soap::envelope;
use super::super::xml::escape;
use super::contract::{self, answer, Answers, DECLARED, INFERRED, THROWN};
use super::json::J;
use super::wsdl_source::SoapChoices;
use super::{
    build, key_of, words_of, Choices, Declared, Endpoint, Param, Source, SourceKind, Status,
};

/// How deep an example body goes.
const DEPTH: usize = 4;

/// An annotation: its simple name and its arguments, each a key (absent
/// for the single unnamed value) and its values, strings resolved.
#[derive(Debug, Clone, Default)]
struct Ann {
    name: String,
    args: Vec<(Option<String>, Vec<String>)>,
}

impl Ann {
    /// The values of `key`, or of the unnamed value when `key` is `value`.
    fn values(&self, keys: &[&str]) -> Vec<String> {
        for (key, values) in &self.args {
            match key {
                None if keys.contains(&"value") => return values.clone(),
                Some(k) if keys.contains(&k.as_str()) => return values.clone(),
                _ => {}
            }
        }
        Vec::new()
    }

    fn first(&self, keys: &[&str]) -> Option<String> {
        self.values(keys).into_iter().next()
    }
}

#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    ty: String,
    anns: Vec<Ann>,
}

#[derive(Debug, Clone, Default)]
struct TypeInfo {
    fields: Vec<FieldInfo>,
    extends: Option<String>,
    /// An enum's constants.
    constants: Vec<String>,
    /// The interfaces it implements, by simple name (FEAT-179).
    implements: Vec<String>,
    /// Its own `@ResponseStatus`: an exception's status.
    status: Option<u16>,
    /// Its methods, to read what they throw.
    methods: Vec<MethodInfo>,
}

/// A method, as far as reading what it answers needs (FEAT-179).
#[derive(Debug, Clone, Default)]
struct MethodInfo {
    name: String,
    returns: String,
    body: String,
    anns: Vec<Ann>,
    params: Vec<String>,
}

/// An `@ExceptionHandler` in a `@ControllerAdvice`: what it catches, and
/// how it answers (FEAT-179).
#[derive(Debug, Clone)]
struct Handler {
    exception: String,
    status: Option<u16>,
    returns: String,
}

/// Every type and string constant the scanned files declare.
#[derive(Default)]
struct World {
    types: HashMap<String, TypeInfo>,
    /// `NAME` and `Class.NAME` to the string it holds.
    strings: HashMap<String, String>,
    /// JAX-RS `@ApplicationPath`.
    application_path: Option<String>,
    /// Every `@ExceptionHandler` of every `@ControllerAdvice`.
    handlers: Vec<Handler>,
}

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

fn children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn last_segment(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// A string literal's contents, `"a" + "b"` joined, and a constant looked
/// up; anything else as written.
fn value_of(node: Node, source: &str, world: &World, class: &str) -> Vec<String> {
    match node.kind() {
        "string_literal" => {
            let raw = text(node, source);
            let inner = raw
                .strip_prefix("\"\"\"")
                .and_then(|r| r.strip_suffix("\"\"\""))
                .or_else(|| raw.strip_prefix('"').and_then(|r| r.strip_suffix('"')))
                .unwrap_or(raw);
            vec![inner.replace("\\\"", "\"").replace("\\\\", "\\")]
        }
        "element_value_array_initializer" | "array_initializer" => children(node)
            .into_iter()
            .flat_map(|child| value_of(child, source, world, class))
            .collect(),
        "binary_expression" => {
            let parts: Vec<String> = children(node)
                .into_iter()
                .map(|child| value_of(child, source, world, class).join(""))
                .collect();
            vec![parts.join("")]
        }
        "parenthesized_expression" => children(node)
            .first()
            .map(|child| value_of(*child, source, world, class))
            .unwrap_or_default(),
        "identifier" | "field_access" | "scoped_identifier" => {
            let raw = text(node, source).to_string();
            let found = world
                .strings
                .get(&raw)
                .or_else(|| world.strings.get(&format!("{class}.{raw}")))
                .or_else(|| world.strings.get(last_segment(&raw)));
            vec![found.cloned().unwrap_or(raw)]
        }
        _ => vec![text(node, source).to_string()],
    }
}

fn annotation(node: Node, source: &str, world: &World, class: &str) -> Option<Ann> {
    let name = node.child_by_field_name("name")?;
    let mut ann = Ann {
        name: last_segment(text(name, source)).to_string(),
        args: Vec::new(),
    };
    if let Some(args) = node.child_by_field_name("arguments") {
        for arg in children(args) {
            if arg.kind() == "element_value_pair" {
                let key = arg
                    .child_by_field_name("key")
                    .map(|k| text(k, source).to_string());
                let values = arg
                    .child_by_field_name("value")
                    .map(|v| value_of(v, source, world, class))
                    .unwrap_or_default();
                ann.args.push((key, values));
            } else {
                ann.args.push((None, value_of(arg, source, world, class)));
            }
        }
    }
    Some(ann)
}

/// The annotations in a declaration's `modifiers`.
fn annotations(node: Node, source: &str, world: &World, class: &str) -> Vec<Ann> {
    children(node)
        .into_iter()
        .filter(|child| child.kind() == "modifiers")
        .flat_map(children)
        .filter(|child| matches!(child.kind(), "annotation" | "marker_annotation"))
        .filter_map(|child| annotation(child, source, world, class))
        .collect()
}

fn has(anns: &[Ann], name: &str) -> bool {
    anns.iter().any(|ann| ann.name == name)
}

fn find<'a>(anns: &'a [Ann], name: &str) -> Option<&'a Ann> {
    anns.iter().find(|ann| ann.name == name)
}

fn is_static(node: Node, source: &str) -> bool {
    children(node)
        .into_iter()
        .filter(|child| child.kind() == "modifiers")
        .any(|modifiers| {
            text(modifiers, source)
                .split_whitespace()
                .any(|w| w == "static")
        })
}

/// A type as written, without its annotations: `@Valid List<Line>` is
/// `List<Line>`.
fn clean_type(ty: &str) -> String {
    let mut out = String::new();
    let mut words = ty.split_whitespace().peekable();
    while let Some(word) = words.next() {
        if word.starts_with('@') {
            // `@Size(max = 3)` may carry spaces: skip to its closing paren.
            if word.contains('(') && !word.contains(')') {
                for rest in words.by_ref() {
                    if rest.contains(')') {
                        break;
                    }
                }
            }
            continue;
        }
        if word == "final" {
            continue;
        }
        out.push_str(word);
    }
    out
}

/// `Map<String, List<Line>>` as `Map` and its arguments.
fn generic(ty: &str) -> (String, Vec<String>) {
    let ty = clean_type(ty);
    let Some(open) = ty.find('<') else {
        return (ty, Vec::new());
    };
    let base = ty[..open].to_string();
    let inner = &ty[open + 1..ty.rfind('>').unwrap_or(ty.len())];
    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                args.push(inner[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    args.push(inner[start..].trim().to_string());
    (base, args)
}

/// The type a body really is: `Mono<ResponseEntity<Order>>` is `Order`.
fn unwrap(ty: &str) -> String {
    let (base, args) = generic(ty);
    match last_segment(&base) {
        "Mono" | "Optional" | "ResponseEntity" | "HttpEntity" | "RequestEntity"
        | "JsonNullable"
            if !args.is_empty() =>
        {
            unwrap(&args[0])
        }
        _ => clean_type(ty),
    }
}

fn number(anns: &[Ann], fallback: &str) -> J {
    for name in ["Min", "DecimalMin"] {
        if let Some(value) = find(anns, name).and_then(|ann| ann.first(&["value"])) {
            let value = value.trim_matches('"').trim_end_matches(['L', 'l']);
            if value.parse::<f64>().is_ok() {
                return J::Num(value.to_string());
            }
        }
    }
    if has(anns, "Positive") {
        return J::Num("1".into());
    }
    J::Num(fallback.into())
}

fn string(field: &str, anns: &[Ann]) -> J {
    let J::Str(text) = shaped(field, anns) else {
        return shaped(field, anns);
    };
    // No longer than `@Size(max)` allows.
    match find(anns, "Size")
        .or_else(|| find(anns, "Length"))
        .and_then(|ann| ann.first(&["max"]))
        .and_then(|v| v.parse::<usize>().ok())
    {
        Some(max) if text.chars().count() > max => J::Str(text.chars().take(max).collect()),
        _ => J::Str(text),
    }
}

fn shaped(field: &str, anns: &[Ann]) -> J {
    let lower = field.to_ascii_lowercase();
    if has(anns, "Email") || lower.contains("email") {
        return J::Str("mira@example.com".into());
    }
    if lower.ends_with("url") || lower.ends_with("uri") {
        return J::Str("https://example.com".into());
    }
    if lower.contains("phone") {
        return J::Str("+15550100".into());
    }
    let min = find(anns, "Size")
        .and_then(|ann| ann.first(&["min"]))
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let sample = if field.is_empty() {
        "string".to_string()
    } else {
        field.to_string()
    };
    if sample.len() < min {
        return J::Str(format!("{sample}{}", "x".repeat(min - sample.len())));
    }
    J::Str(sample)
}

/// A sample of type `ty` for the field `field`.
fn sample(
    ty: &str,
    field: &str,
    anns: &[Ann],
    world: &World,
    depth: usize,
    seen: &mut Vec<String>,
) -> J {
    let ty = clean_type(ty);
    if let Some(element) = ty.strip_suffix("[]") {
        if element == "byte" {
            return J::Str("base64".into());
        }
        return J::Arr(vec![sample(element, field, anns, world, depth, seen)]);
    }
    let (base, args) = generic(&ty);
    let arg = |at: usize| args.get(at).cloned().unwrap_or_else(|| "Object".into());
    match last_segment(&base) {
        "String" | "CharSequence" => string(field, anns),
        "char" | "Character" => J::Str("a".into()),
        "int" | "Integer" | "long" | "Long" | "short" | "Short" | "byte" | "Byte"
        | "BigInteger" => number(anns, "0"),
        "double" | "Double" | "float" | "Float" | "BigDecimal" | "Number" => number(anns, "0.0"),
        "boolean" | "Boolean" => J::Bool(false),
        "UUID" => J::Str("3fa85f64-5717-4562-b3fc-2c963f66afa6".into()),
        "LocalDate" => J::Str("2026-01-31".into()),
        "LocalDateTime" => J::Str("2026-01-31T09:30:00".into()),
        "Instant" | "OffsetDateTime" | "ZonedDateTime" | "Date" | "Timestamp" => {
            J::Str("2026-01-31T09:30:00Z".into())
        }
        "LocalTime" => J::Str("09:30:00".into()),
        "Duration" => J::Str("PT30M".into()),
        "List" | "Set" | "Collection" | "Iterable" | "ArrayList" | "LinkedList" | "HashSet"
        | "SortedSet" | "Flux" => J::Arr(vec![sample(&arg(0), field, &[], world, depth, seen)]),
        "Map" | "HashMap" | "LinkedHashMap" | "TreeMap" => J::Obj(vec![(
            "key".into(),
            sample(&arg(1), "", &[], world, depth, seen),
        )]),
        "Optional" | "Mono" | "JsonNullable" => sample(&arg(0), field, anns, world, depth, seen),
        "Object" | "JsonNode" | "ObjectNode" => J::Obj(Vec::new()),
        name => match world.types.get(name) {
            Some(info) if !info.constants.is_empty() => J::Str(info.constants[0].clone()),
            Some(_) if depth >= DEPTH || seen.iter().any(|s| s == name) => J::Null,
            Some(_) => {
                seen.push(name.to_string());
                let out = object(name, world, depth + 1, seen);
                seen.pop();
                out
            }
            None => J::Null,
        },
    }
}

/// A class or record's fields as an object, its parents' first.
fn object(name: &str, world: &World, depth: usize, seen: &mut Vec<String>) -> J {
    let mut fields = Vec::new();
    let mut chain = Vec::new();
    let mut at = Some(name.to_string());
    while let Some(current) = at {
        if chain.contains(&current) {
            break;
        }
        let Some(info) = world.types.get(&current) else {
            break;
        };
        chain.push(current);
        at = info.extends.clone();
    }
    for class in chain.iter().rev() {
        for field in &world.types[class].fields {
            if has(&field.anns, "JsonIgnore") {
                continue;
            }
            let key = find(&field.anns, "JsonProperty")
                .and_then(|ann| ann.first(&["value"]))
                .unwrap_or_else(|| field.name.clone());
            let value = sample(&field.ty, &field.name, &field.anns, world, depth, seen);
            fields.push((key, value));
        }
    }
    J::Obj(fields)
}

struct Parsed {
    path: String,
    text: String,
    tree: tree_sitter::Tree,
}

fn type_declarations<'t>(node: Node<'t>, out: &mut Vec<Node<'t>>) {
    for child in children(node) {
        match child.kind() {
            "class_declaration"
            | "record_declaration"
            | "interface_declaration"
            | "enum_declaration" => {
                out.push(child);
                if let Some(body) = child.child_by_field_name("body") {
                    type_declarations(body, out);
                }
            }
            _ => {}
        }
    }
}

/// The interfaces a class says it implements, by simple name.
fn interfaces_of(decl: Node, source: &str) -> Vec<String> {
    let Some(list) = decl.child_by_field_name("interfaces") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut stack = vec![list];
    while let Some(node) = stack.pop() {
        for child in children(node) {
            match child.kind() {
                "type_identifier" | "generic_type" | "scoped_type_identifier" => {
                    out.push(last_segment(&generic(text(child, source)).0).to_string())
                }
                _ => stack.push(child),
            }
        }
    }
    out
}

/// A type's methods, with their bodies as text.
fn methods_of(decl: Node, source: &str, world: &World, class: &str) -> Vec<MethodInfo> {
    decl.child_by_field_name("body")
        .map(children)
        .unwrap_or_default()
        .into_iter()
        .filter(|member| member.kind() == "method_declaration")
        .map(|member| MethodInfo {
            name: member
                .child_by_field_name("name")
                .map(|n| text(n, source).to_string())
                .unwrap_or_default(),
            returns: member
                .child_by_field_name("type")
                .map(|t| text(t, source).to_string())
                .unwrap_or_default(),
            body: member
                .child_by_field_name("body")
                .map(|b| text(b, source).to_string())
                .unwrap_or_default(),
            anns: annotations(member, source, world, class),
            params: member
                .child_by_field_name("parameters")
                .map(children)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| p.child_by_field_name("type"))
                .map(|t| clean_type(text(t, source)))
                .collect(),
        })
        .collect()
}

/// The status a method's body sets, the first it names.
fn status_in(body: &str) -> Option<u16> {
    let named =
        regex::Regex::new(r"(?:HttpStatus|Response\.Status)\.([A-Z_]+)").expect("a valid pattern");
    if let Some(found) = named.captures(body) {
        return contract::status_of(&found[1]);
    }
    for (call, status) in [
        ("ResponseEntity.notFound()", 404),
        ("ResponseEntity.badRequest()", 400),
        ("ResponseEntity.unprocessableEntity()", 422),
        ("ResponseEntity.internalServerError()", 500),
    ] {
        if body.contains(call) {
            return Some(status);
        }
    }
    None
}

/// What every file declares: types, string constants, `@ApplicationPath`.
fn index(files: &[Parsed]) -> World {
    let mut world = World::default();
    // Constants first, in two passes, so one constant may name another.
    for _ in 0..2 {
        for file in files {
            let mut decls = Vec::new();
            type_declarations(file.tree.root_node(), &mut decls);
            for decl in decls {
                let class = decl
                    .child_by_field_name("name")
                    .map(|n| text(n, &file.text).to_string())
                    .unwrap_or_default();
                let Some(body) = decl.child_by_field_name("body") else {
                    continue;
                };
                for member in children(body) {
                    if member.kind() != "field_declaration"
                        && member.kind() != "constant_declaration"
                    {
                        continue;
                    }
                    let ty = member
                        .child_by_field_name("type")
                        .map(|t| text(t, &file.text))
                        .unwrap_or("");
                    let constant =
                        is_static(member, &file.text) || decl.kind() == "interface_declaration";
                    if ty != "String" || !constant {
                        continue;
                    }
                    let mut cursor = member.walk();
                    for declarator in member.children_by_field_name("declarator", &mut cursor) {
                        let (Some(name), Some(value)) = (
                            declarator.child_by_field_name("name"),
                            declarator.child_by_field_name("value"),
                        ) else {
                            continue;
                        };
                        let name = text(name, &file.text).to_string();
                        let resolved = value_of(value, &file.text, &world, &class).join("");
                        world
                            .strings
                            .insert(format!("{class}.{name}"), resolved.clone());
                        world.strings.entry(name).or_insert(resolved);
                    }
                }
            }
        }
    }
    let mut handlers: Vec<Handler> = Vec::new();
    for file in files {
        let mut decls = Vec::new();
        type_declarations(file.tree.root_node(), &mut decls);
        for decl in decls {
            let Some(name) = decl
                .child_by_field_name("name")
                .map(|n| text(n, &file.text).to_string())
            else {
                continue;
            };
            let anns = annotations(decl, &file.text, &world, &name);
            if let Some(path) = find(&anns, "ApplicationPath").and_then(|a| a.first(&["value"])) {
                world.application_path = Some(path);
            }
            let mut info = TypeInfo::default();
            match decl.kind() {
                "record_declaration" => {
                    if let Some(params) = decl.child_by_field_name("parameters") {
                        for param in children(params) {
                            if let (Some(ty), Some(field)) = (
                                param.child_by_field_name("type"),
                                param.child_by_field_name("name"),
                            ) {
                                info.fields.push(FieldInfo {
                                    name: text(field, &file.text).to_string(),
                                    ty: text(ty, &file.text).to_string(),
                                    anns: annotations(param, &file.text, &world, &name),
                                });
                            }
                        }
                    }
                }
                "enum_declaration" => {
                    if let Some(body) = decl.child_by_field_name("body") {
                        info.constants = children(body)
                            .into_iter()
                            .filter(|c| c.kind() == "enum_constant")
                            .filter_map(|c| c.child_by_field_name("name"))
                            .map(|n| text(n, &file.text).to_string())
                            .collect();
                    }
                }
                "class_declaration" => {
                    info.extends = decl
                        .child_by_field_name("superclass")
                        .and_then(|s| children(s).into_iter().next())
                        .map(|t| generic(text(t, &file.text)).0)
                        .map(|t| last_segment(&t).to_string());
                    if let Some(body) = decl.child_by_field_name("body") {
                        for member in children(body) {
                            if member.kind() != "field_declaration" || is_static(member, &file.text)
                            {
                                continue;
                            }
                            if children(member).into_iter().any(|c| {
                                c.kind() == "modifiers" && text(c, &file.text).contains("transient")
                            }) {
                                continue;
                            }
                            let ty = member
                                .child_by_field_name("type")
                                .map(|t| text(t, &file.text).to_string())
                                .unwrap_or_default();
                            let anns = annotations(member, &file.text, &world, &name);
                            let mut cursor = member.walk();
                            for declarator in
                                member.children_by_field_name("declarator", &mut cursor)
                            {
                                if let Some(field) = declarator.child_by_field_name("name") {
                                    info.fields.push(FieldInfo {
                                        name: text(field, &file.text).to_string(),
                                        ty: ty.clone(),
                                        anns: anns.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
            // What it does, for what an endpoint answers (FEAT-179).
            info.implements = interfaces_of(decl, &file.text);
            info.status = find(&anns, "ResponseStatus")
                .and_then(|a| a.first(&["value", "code"]))
                .and_then(|s| contract::status_of(&s));
            info.methods = methods_of(decl, &file.text, &world, &name);
            if has(&anns, "ControllerAdvice") || has(&anns, "RestControllerAdvice") {
                for method in &info.methods {
                    let Some(handles) = find(&method.anns, "ExceptionHandler") else {
                        continue;
                    };
                    let mut caught: Vec<String> = handles
                        .values(&["value"])
                        .iter()
                        .map(|v| last_segment(v.trim_end_matches(".class")).to_string())
                        .filter(|v| !v.is_empty())
                        .collect();
                    if caught.is_empty() {
                        caught = method
                            .params
                            .iter()
                            .filter(|p| p.ends_with("Exception") || p.ends_with("Error"))
                            .map(|p| last_segment(p).to_string())
                            .collect();
                    }
                    let status = find(&method.anns, "ResponseStatus")
                        .and_then(|a| a.first(&["value", "code"]))
                        .and_then(|s| contract::status_of(&s))
                        .or_else(|| status_in(&method.body));
                    for exception in caught {
                        handlers.push(Handler {
                            exception,
                            status,
                            returns: method.returns.clone(),
                        });
                    }
                }
            }
            world.types.entry(name).or_insert(info);
        }
    }
    world.handlers = handlers;
    world
}

/// JAX-RS's own exceptions, and the status each sends.
const JAXRS_EXCEPTIONS: [(&str, u16); 8] = [
    ("BadRequestException", 400),
    ("NotAuthorizedException", 401),
    ("ForbiddenException", 403),
    ("NotFoundException", 404),
    ("NotAllowedException", 405),
    ("NotAcceptableException", 406),
    ("InternalServerErrorException", 500),
    ("ServiceUnavailableException", 503),
];

/// The rules the fields of `ty` are held to, a field per path, its parents'
/// fields first (FEAT-179).
fn rules_of(ty: &str, world: &World) -> Vec<FieldRule> {
    let mut out = Vec::new();
    walk_rules(ty, "", world, 0, &mut Vec::new(), &mut out);
    out
}

fn walk_rules(
    ty: &str,
    prefix: &str,
    world: &World,
    depth: usize,
    seen: &mut Vec<String>,
    out: &mut Vec<FieldRule>,
) {
    let ty = clean_type(ty);
    let (base, args) = generic(&ty);
    let simple = last_segment(&base).to_string();
    if is_list(&simple) || ty.ends_with("[]") {
        let element = args
            .first()
            .cloned()
            .unwrap_or_else(|| ty.trim_end_matches("[]").to_string());
        return walk_rules(&element, &format!("{prefix}[]"), world, depth, seen, out);
    }
    if matches!(simple.as_str(), "Optional" | "JsonNullable" | "Mono") {
        if let Some(inner) = args.first() {
            return walk_rules(inner, prefix, world, depth, seen, out);
        }
    }
    let Some(info) = world.types.get(&simple) else {
        return;
    };
    if !info.constants.is_empty() || depth >= DEPTH || seen.contains(&simple) {
        return;
    }
    seen.push(simple.clone());
    let mut chain = Vec::new();
    let mut at = Some(simple.clone());
    while let Some(current) = at {
        if chain.contains(&current) {
            break;
        }
        let Some(info) = world.types.get(&current) else {
            break;
        };
        chain.push(current);
        at = info.extends.clone();
    }
    for class in chain.iter().rev() {
        for field in &world.types[class].fields {
            if has(&field.anns, "JsonIgnore") {
                continue;
            }
            let key = find(&field.anns, "JsonProperty")
                .and_then(|ann| ann.first(&["value"]))
                .unwrap_or_else(|| field.name.clone());
            let path = if prefix.is_empty() {
                key
            } else {
                format!("{prefix}.{key}")
            };
            out.push(rule_of(&path, &field.ty, &field.anns, world));
            walk_rules(&field.ty, &path, world, depth + 1, seen, out);
        }
    }
    seen.pop();
}

fn is_list(name: &str) -> bool {
    matches!(
        name,
        "List"
            | "Set"
            | "Collection"
            | "Iterable"
            | "ArrayList"
            | "LinkedList"
            | "HashSet"
            | "SortedSet"
            | "Flux"
    )
}

/// A Bean Validation bound as a number: `5`, `"0.01"`, `10L`.
fn bound(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_matches('"')
        .trim_end_matches(['L', 'l'])
        .parse::<f64>()
        .ok()
}

/// One field's rule: its type, and what its annotations hold it to.
fn rule_of(path: &str, ty: &str, anns: &[Ann], world: &World) -> FieldRule {
    let ty = clean_type(ty);
    let (base, args) = generic(&ty);
    let simple = last_segment(&base);
    let (kind, format, values): (&str, Option<&str>, Vec<String>) =
        if ty.ends_with("[]") && simple != "byte[]" {
            ("array", None, Vec::new())
        } else {
            match simple {
                "String" | "CharSequence" | "char" | "Character" | "byte[]" => {
                    ("string", None, Vec::new())
                }
                "int" | "Integer" | "long" | "Long" | "short" | "Short" | "byte" | "Byte"
                | "BigInteger" => ("integer", None, Vec::new()),
                "double" | "Double" | "float" | "Float" | "BigDecimal" | "Number" => {
                    ("number", None, Vec::new())
                }
                "boolean" | "Boolean" => ("boolean", None, Vec::new()),
                "UUID" => ("string", Some("uuid"), Vec::new()),
                "LocalDate" => ("string", Some("date"), Vec::new()),
                "LocalDateTime" | "Instant" | "OffsetDateTime" | "ZonedDateTime" | "Date"
                | "Timestamp" => ("string", Some("date-time"), Vec::new()),
                "LocalTime" => ("string", Some("time"), Vec::new()),
                "Optional" | "JsonNullable" => {
                    let inner = rule_of(
                        path,
                        args.first().map(String::as_str).unwrap_or("Object"),
                        anns,
                        world,
                    );
                    return FieldRule {
                        required: false,
                        ..inner
                    };
                }
                name if is_list(name) => ("array", None, Vec::new()),
                name => match world.types.get(name) {
                    Some(info) if !info.constants.is_empty() => {
                        ("string", None, info.constants.clone())
                    }
                    _ => ("object", None, Vec::new()),
                },
            }
        };
    let mut rule = FieldRule {
        path: path.to_string(),
        kind: kind.to_string(),
        required: has(anns, "NotNull") || has(anns, "NotBlank") || has(anns, "NotEmpty"),
        format: format.map(str::to_string),
        values,
        ..FieldRule::default()
    };
    if kind == "string" && (has(anns, "NotBlank") || has(anns, "NotEmpty")) {
        rule.min_length = Some(1);
    }
    if let Some(size) = find(anns, "Size").or_else(|| find(anns, "Length")) {
        if kind == "string" {
            if let Some(min) = size.first(&["min"]).and_then(|v| bound(&v)) {
                rule.min_length = Some(min as u64);
            }
            if let Some(max) = size.first(&["max"]).and_then(|v| bound(&v)) {
                rule.max_length = Some(max as u64);
            }
        }
    }
    for name in ["Min", "DecimalMin"] {
        if let Some(value) = find(anns, name)
            .and_then(|a| a.first(&["value"]))
            .and_then(|v| bound(&v))
        {
            rule.minimum = Some(value);
        }
    }
    for name in ["Max", "DecimalMax"] {
        if let Some(value) = find(anns, name)
            .and_then(|a| a.first(&["value"]))
            .and_then(|v| bound(&v))
        {
            rule.maximum = Some(value);
        }
    }
    if has(anns, "Positive") && kind == "integer" {
        rule.minimum = Some(1.0);
    }
    if has(anns, "PositiveOrZero") {
        rule.minimum = Some(0.0);
    }
    if has(anns, "Negative") && kind == "integer" {
        rule.maximum = Some(-1.0);
    }
    if has(anns, "NegativeOrZero") {
        rule.maximum = Some(0.0);
    }
    if has(anns, "Email") {
        rule.format = Some("email".into());
    }
    if let Some(pattern) = find(anns, "Pattern").and_then(|a| a.first(&["regexp"])) {
        rule.pattern = Some(pattern);
    }
    rule
}

/// The example a method's return type gives, or none when it says nothing:
/// `ResponseEntity<Order>` is an order, `ResponseEntity<Void>` nothing.
fn returned(returns: &str, world: &World) -> Option<String> {
    let mut ty = clean_type(returns);
    loop {
        let (base, args) = generic(&ty);
        match last_segment(&base) {
            "ResponseEntity" | "HttpEntity" | "Mono" | "Optional" | "CompletableFuture"
            | "CompletionStage" | "DeferredResult" | "Callable" | "Future"
                if !args.is_empty() =>
            {
                ty = args[0].clone()
            }
            _ => break,
        }
    }
    if matches!(
        last_segment(&generic(&ty).0),
        "void" | "Void" | "?" | "Object" | "Response" | "ResponseEntity" | "HttpEntity"
    ) {
        return None;
    }
    let example = sample(&ty, "", &[], world, 0, &mut Vec::new());
    (!matches!(example, J::Null)).then(|| example.pretty())
}

/// The text inside each `@<name>(…)` in `text`, parentheses balanced.
fn annotation_bodies(text: &str, name: &str) -> Vec<String> {
    let open = format!("@{name}(");
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(&open) {
        let inner = &rest[at + open.len()..];
        let mut depth = 1;
        let mut end = inner.len();
        for (index, c) in inner.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = index;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push(inner[..end].to_string());
        rest = &inner[end..];
    }
    out
}

/// springdoc's `@ApiResponse(responseCode = "404", description = "…")`s.
fn api_responses(text: &str) -> Vec<(u16, Option<String>)> {
    let code = regex::Regex::new(r#"responseCode\s*=\s*"(\d{3})""#).expect("a valid pattern");
    let words = regex::Regex::new(r#"description\s*=\s*"([^"]*)""#).expect("a valid pattern");
    annotation_bodies(text, "ApiResponse")
        .iter()
        .filter_map(|inner| {
            let status = code.captures(inner)?[1].parse::<u16>().ok()?;
            Some((
                status,
                words
                    .captures(inner)
                    .map(|m| m[1].to_string())
                    .filter(|w| !w.is_empty()),
            ))
        })
        .collect()
}

/// The success a method's body builds: `ResponseEntity.created(…)` is 201.
fn success_in(body: &str) -> Option<u16> {
    let created = [
        "ResponseEntity.created(",
        "Response.created(",
        "HttpStatus.CREATED",
        "Response.Status.CREATED",
    ];
    let empty = [
        "ResponseEntity.noContent()",
        "Response.noContent()",
        "HttpStatus.NO_CONTENT",
        "Response.Status.NO_CONTENT",
    ];
    let accepted = [
        "ResponseEntity.accepted()",
        "Response.accepted(",
        "HttpStatus.ACCEPTED",
        "Response.Status.ACCEPTED",
    ];
    if created.iter().any(|c| body.contains(c)) {
        Some(201)
    } else if empty.iter().any(|c| body.contains(c)) {
        Some(204)
    } else if accepted.iter().any(|c| body.contains(c)) {
        Some(202)
    } else {
        None
    }
}

/// Failures a method's body answers itself: `new ResponseStatusException(
/// HttpStatus.CONFLICT, "…")`, `ResponseEntity.notFound()`,
/// `new WebApplicationException(Response.Status.GONE)`.
fn failures_in(body: &str) -> Vec<(u16, Option<String>)> {
    let raised = regex::Regex::new(
        r#"new\s+(?:ResponseStatusException|WebApplicationException)\s*\(\s*(?:"([^"]*)"\s*,\s*)?(?:HttpStatus|Response\.Status)\.([A-Z_]+)\s*(?:,\s*"([^"]*)")?"#,
    )
    .expect("a valid pattern");
    let built = regex::Regex::new(
        r"(?:ResponseEntity|Response)\.status\(\s*(?:HttpStatus|Response\.Status)\.([A-Z_]+)",
    )
    .expect("a valid pattern");
    let mut out: Vec<(u16, Option<String>)> = Vec::new();
    let mut push = |status: Option<u16>, message: Option<String>| {
        if let Some(status) = status.filter(|s| *s >= 400) {
            if !out.iter().any(|(seen, _)| *seen == status) {
                out.push((status, message));
            }
        }
    };
    for found in raised.captures_iter(body) {
        let message = found
            .get(3)
            .or_else(|| found.get(1))
            .map(|m| m.as_str().to_string());
        push(contract::status_of(&found[2]), message);
    }
    for found in built.captures_iter(body) {
        push(contract::status_of(&found[1]), None);
    }
    for (call, status) in [
        ("ResponseEntity.notFound()", 404),
        ("ResponseEntity.badRequest()", 400),
        ("ResponseEntity.unprocessableEntity()", 422),
    ] {
        if body.contains(call) {
            push(Some(status), None);
        }
    }
    out
}

/// `/api` and `orders/` as `/api/orders`.
fn join(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        let part = part.trim().trim_matches('/');
        if !part.is_empty() {
            out.push('/');
            out.push_str(part);
        }
    }
    if out.is_empty() {
        "/".into()
    } else {
        out
    }
}

const SPRING_MAPPINGS: [(&str, &str); 5] = [
    ("GetMapping", "GET"),
    ("PostMapping", "POST"),
    ("PutMapping", "PUT"),
    ("PatchMapping", "PATCH"),
    ("DeleteMapping", "DELETE"),
];
const JAXRS_METHODS: [&str; 7] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

/// The Spring mapping on a method: its method and paths.
fn spring_mapping(anns: &[Ann]) -> Option<(String, Vec<String>, &Ann)> {
    for (name, method) in SPRING_MAPPINGS {
        if let Some(ann) = find(anns, name) {
            return Some((method.to_string(), ann.values(&["value", "path"]), ann));
        }
    }
    let ann = find(anns, "RequestMapping")?;
    let method = ann
        .first(&["method"])
        .map(|m| last_segment(&m).to_ascii_uppercase())
        .unwrap_or_else(|| "GET".into());
    Some((method, ann.values(&["value", "path"]), ann))
}

struct Endpoints<'a> {
    world: &'a World,
    frameworks: &'a [String],
    soap: &'a SoapChoices,
    out: Vec<Endpoint>,
    sources: Vec<Source>,
}

impl Endpoints<'_> {
    fn file(&mut self, file: &Parsed) {
        let mut decls = Vec::new();
        type_declarations(file.tree.root_node(), &mut decls);
        let reactive = file.text.contains("reactor.core.publisher");
        let start = self.out.len();
        let mut kind = SourceKind::Spring;
        for decl in decls {
            if !matches!(decl.kind(), "class_declaration" | "interface_declaration") {
                continue;
            }
            let Some(class) = decl
                .child_by_field_name("name")
                .map(|n| text(n, &file.text).to_string())
            else {
                continue;
            };
            let anns = annotations(decl, &file.text, self.world, &class);
            // The loops below name each method's own `anns`; the class's are these.
            let class_anns = anns.clone();
            let methods: Vec<Node> = decl
                .child_by_field_name("body")
                .map(children)
                .unwrap_or_default()
                .into_iter()
                .filter(|m| m.kind() == "method_declaration")
                .collect();
            let method_anns: Vec<Vec<Ann>> = methods
                .iter()
                .map(|m| annotations(*m, &file.text, self.world, &class))
                .collect();

            let spring_class = has(&anns, "RestController")
                || (has(&anns, "Controller")
                    && (has(&anns, "ResponseBody")
                        || method_anns.iter().any(|a| has(a, "ResponseBody"))))
                || (decl.kind() == "interface_declaration"
                    && method_anns.iter().any(|a| spring_mapping(a).is_some()));
            let jaxrs_class = !spring_class
                && (has(&anns, "Path")
                    || method_anns
                        .iter()
                        .any(|a| JAXRS_METHODS.iter().any(|m| has(a, m))));
            let frameworks = self.frameworks;
            let wanted = |name: &str| frameworks.iter().any(|f| f == name);

            if spring_class
                && (wanted(if reactive {
                    "springWebflux"
                } else {
                    "springMvc"
                }))
            {
                let base = find(&anns, "RequestMapping")
                    .and_then(|a| a.values(&["value", "path"]).into_iter().next())
                    .unwrap_or_default();
                let class_consumes =
                    find(&anns, "RequestMapping").and_then(|a| a.first(&["consumes"]));
                for (method, anns) in methods.iter().zip(&method_anns) {
                    let Some((verb, paths, mapping)) = spring_mapping(anns) else {
                        continue;
                    };
                    let path = join(&[&base, paths.first().map(String::as_str).unwrap_or("")]);
                    let consumes = mapping
                        .first(&["consumes"])
                        .or_else(|| class_consumes.clone());
                    self.method(
                        file,
                        &class,
                        &class_anns,
                        *method,
                        verb,
                        path,
                        consumes,
                        false,
                    );
                }
            } else if jaxrs_class && wanted("jaxRs") {
                kind = SourceKind::JaxRs;
                let base = find(&anns, "Path")
                    .and_then(|a| a.first(&["value"]))
                    .unwrap_or_default();
                let app = self.world.application_path.clone().unwrap_or_default();
                let class_consumes = find(&anns, "Consumes").and_then(|a| a.first(&["value"]));
                for (method, anns) in methods.iter().zip(&method_anns) {
                    let Some(verb) = JAXRS_METHODS.iter().find(|m| has(anns, m)) else {
                        continue;
                    };
                    let own = find(anns, "Path")
                        .and_then(|a| a.first(&["value"]))
                        .unwrap_or_default();
                    let path = join(&[&app, &base, &own]);
                    let consumes = find(anns, "Consumes")
                        .and_then(|a| a.first(&["value"]))
                        .or_else(|| class_consumes.clone())
                        .map(|c| {
                            last_segment(&c)
                                .replace("APPLICATION_JSON", "application/json")
                                .replace(
                                    "APPLICATION_FORM_URLENCODED",
                                    "application/x-www-form-urlencoded",
                                )
                                .replace("_TYPE", "")
                        });
                    self.method(
                        file,
                        &class,
                        &class_anns,
                        *method,
                        verb.to_string(),
                        path,
                        consumes,
                        true,
                    );
                }
            } else if has(&anns, "WebService") && wanted("jaxWs") {
                kind = SourceKind::JaxWs;
                let interface = decl.kind() == "interface_declaration";
                self.jaxws(file, &class, &anns, &methods, &method_anns, interface);
            } else if has(&anns, "Endpoint") && wanted("springWs") {
                kind = SourceKind::SpringWs;
                self.spring_ws(file, &class, &methods, &method_anns);
            }
        }
        let count = self.out.len() - start;
        if count > 0 {
            let label = file
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&file.path)
                .to_string();
            self.sources.push(Source {
                id: format!("code:{}", file.path),
                kind,
                label,
                file: Some(file.path.clone()),
                count,
                error: None,
                in_repository: true,
            });
        }
    }

    /// The type an exception extends, and so on up, itself first.
    fn lineage(&self, exception: &str) -> Vec<String> {
        let mut out = vec![exception.to_string()];
        while let Some(parent) = self
            .world
            .types
            .get(out.last().expect("one"))
            .and_then(|t| t.extends.clone())
        {
            if out.contains(&parent) {
                break;
            }
            out.push(parent);
        }
        out
    }

    /// The `@ExceptionHandler` that catches `exception`, nearest first.
    fn handler(&self, exception: &str) -> Option<&Handler> {
        self.lineage(exception)
            .iter()
            .find_map(|name| self.world.handlers.iter().find(|h| h.exception == *name))
    }

    /// The status `exception` is answered with: its handler's, its own
    /// `@ResponseStatus`, JAX-RS's own, then what its name says when it is
    /// the code's own exception. None is a 500.
    fn exception_status(&self, exception: &str) -> Option<u16> {
        let lineage = self.lineage(exception);
        if let Some(status) = self.handler(exception).and_then(|h| h.status) {
            return Some(status);
        }
        for name in &lineage {
            if let Some(status) = self.world.types.get(name).and_then(|t| t.status) {
                return Some(status);
            }
            if let Some((_, status)) = JAXRS_EXCEPTIONS.iter().find(|(known, _)| known == name) {
                return Some(*status);
            }
        }
        if !self.world.types.contains_key(exception) {
            return None;
        }
        lineage
            .iter()
            .find_map(|name| contract::status_by_name(name))
    }

    /// What a failure's body looks like: the handler's answer when one
    /// catches it, else what Spring Boot sends (JAX-RS sends nothing set).
    fn error_body(
        &self,
        exceptions: &[&str],
        status: u16,
        path: &str,
        jaxrs: bool,
    ) -> Option<String> {
        let returns = exceptions
            .iter()
            .find_map(|e| self.handler(e))
            .map(|h| h.returns.clone());
        if let Some(returns) = returns {
            let inner = unwrap(&returns);
            if last_segment(&generic(&inner).0) == "ProblemDetail" {
                return Some(
                    J::Obj(vec![
                        ("type".into(), J::Str("about:blank".into())),
                        ("title".into(), J::Str(contract::reason(status).into())),
                        ("status".into(), J::Num(status.to_string())),
                        ("detail".into(), J::Str(String::new())),
                        ("instance".into(), J::Str(path.into())),
                    ])
                    .pretty(),
                );
            }
            let example = sample(&inner, "", &[], self.world, 0, &mut Vec::new());
            if !matches!(example, J::Null) {
                return Some(example.pretty());
            }
        }
        (!jaxrs).then(|| contract::spring_error(status, path))
    }

    /// The exceptions a service the controller holds throws, from the one
    /// method `call` names on its field `field`.
    fn thrown_by(&self, class: &str, field: &str, call: &str) -> Vec<(String, Option<String>)> {
        let Some(held) = self
            .world
            .types
            .get(class)
            .and_then(|info| info.fields.iter().find(|f| f.name == field))
            .map(|f| last_segment(&generic(&f.ty).0).to_string())
        else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (name, info) in &self.world.types {
            if *name != held && !info.implements.contains(&held) {
                continue;
            }
            for method in info.methods.iter().filter(|m| m.name == call) {
                for found in contract::thrown(&method.body) {
                    if !out
                        .iter()
                        .any(|(seen, _): &(String, Option<String>)| *seen == found.0)
                    {
                        out.push(found);
                    }
                }
            }
        }
        out
    }

    /// What a method answers (FEAT-179).
    #[allow(clippy::too_many_arguments)]
    fn answers(
        &self,
        source: &str,
        class: &str,
        class_anns: &[Ann],
        method: Node,
        path: &str,
        validated: bool,
        required: bool,
        jaxrs: bool,
    ) -> Vec<Answer> {
        let anns = annotations(method, source, self.world, class);
        let body = method
            .child_by_field_name("body")
            .map(|b| text(b, source))
            .unwrap_or("");
        let returns = method
            .child_by_field_name("type")
            .map(|t| text(t, source))
            .unwrap_or("void");
        let mut answers = Answers::default();

        let declared = find(&anns, "ResponseStatus")
            .and_then(|a| a.first(&["value", "code"]))
            .and_then(|s| contract::status_of(&s));
        let success = declared.or_else(|| success_in(body)).unwrap_or(200);
        let example = if success == 204 {
            None
        } else {
            returned(returns, self.world)
        };
        answers.add(
            INFERRED,
            answer(
                success,
                contract::success_words(success),
                example,
                if declared.is_some() {
                    "@ResponseStatus"
                } else {
                    "the return type"
                },
            ),
        );

        if validated {
            answers.add(
                INFERRED,
                answer(
                    400,
                    "The body breaks one of its rules",
                    self.error_body(
                        &["MethodArgumentNotValidException", "BindException"],
                        400,
                        path,
                        jaxrs,
                    ),
                    "@Valid",
                ),
            );
        } else if required {
            answers.add(
                INFERRED,
                answer(
                    400,
                    "A required parameter is missing",
                    self.error_body(
                        &["MissingServletRequestParameterException"],
                        400,
                        path,
                        jaxrs,
                    ),
                    "a required parameter",
                ),
            );
        }

        let guard = ["PreAuthorize", "Secured", "RolesAllowed"]
            .iter()
            .find_map(|name| find(&anns, name).or_else(|| find(class_anns, name)));
        if let Some(guard) = guard {
            let rule = guard.first(&["value"]).unwrap_or_default();
            let from = format!("@{}", guard.name);
            answers.add(
                INFERRED,
                answer(
                    401,
                    "Not signed in",
                    self.error_body(&[], 401, path, jaxrs),
                    from.clone(),
                ),
            );
            let says = if rule.is_empty() {
                "Signed in without the right to".to_string()
            } else {
                format!("Signed in, but not {rule}")
            };
            answers.add(
                INFERRED,
                answer(
                    403,
                    says,
                    self.error_body(&["AccessDeniedException"], 403, path, jaxrs),
                    from,
                ),
            );
        }

        let mut thrown = contract::thrown(body);
        for (field, call) in contract::calls_on_fields(body) {
            for found in self.thrown_by(class, &field, &call) {
                if !thrown.iter().any(|(seen, _)| *seen == found.0) {
                    thrown.push(found);
                }
            }
        }
        for (name, message) in thrown {
            if let Some(status) = self.exception_status(&name) {
                let says = message.unwrap_or_else(|| contract::sentence(&name));
                answers.add(
                    THROWN,
                    answer(
                        status,
                        says,
                        self.error_body(&[&name], status, path, jaxrs),
                        format!("throws {name}"),
                    ),
                );
            }
        }
        for (status, message) in failures_in(body) {
            let says = message.unwrap_or_else(|| contract::reason(status).to_string());
            answers.add(
                THROWN,
                answer(
                    status,
                    says,
                    self.error_body(&[], status, path, jaxrs),
                    "the method’s body",
                ),
            );
        }

        for (status, words) in api_responses(text(method, source)) {
            let says = words.unwrap_or_else(|| contract::reason(status).to_string());
            answers.add(DECLARED, answer(status, says, None, "@ApiResponse"));
        }

        answers.add(
            INFERRED,
            answer(
                500,
                "Something failed on the server",
                self.error_body(&["Exception"], 500, path, jaxrs),
                "any endpoint",
            ),
        );
        answers.done()
    }

    #[allow(clippy::too_many_arguments)]
    fn method(
        &mut self,
        file: &Parsed,
        class: &str,
        class_anns: &[Ann],
        method: Node,
        verb: String,
        path: String,
        consumes: Option<String>,
        jaxrs: bool,
    ) {
        let source = &file.text;
        let name = method
            .child_by_field_name("name")
            .map(|n| text(n, source).to_string())
            .unwrap_or_default();
        let mut declared = Declared {
            method: verb.clone(),
            path: path.clone(),
            consumes: consumes.clone(),
            ..Declared::default()
        };
        let mut form: Vec<(String, J)> = Vec::new();
        let mut multipart = false;
        // The body's type, whether it is validated, whether a parameter must be given (FEAT-179).
        let mut body_ty: Option<String> = None;
        let mut validated = false;
        let mut required = false;
        if let Some(params) = method.child_by_field_name("parameters") {
            for param in children(params) {
                if param.kind() != "formal_parameter" {
                    continue;
                }
                let anns = annotations(param, source, self.world, class);
                let ty = param
                    .child_by_field_name("type")
                    .map(|t| text(t, source))
                    .unwrap_or("");
                let pname = param
                    .child_by_field_name("name")
                    .map(|n| text(n, source).to_string())
                    .unwrap_or_default();
                let named = |ann: &Ann| {
                    ann.first(&["value", "name"])
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| pname.clone())
                };
                if jaxrs {
                    let default = find(&anns, "DefaultValue")
                        .and_then(|a| a.first(&["value"]))
                        .unwrap_or_default();
                    if let Some(ann) = find(&anns, "QueryParam") {
                        declared.query.push(Param {
                            name: named(ann),
                            sample: default,
                            required: false,
                        });
                    } else if let Some(ann) = find(&anns, "HeaderParam") {
                        declared.headers.push(Param {
                            name: named(ann),
                            sample: default,
                            required: false,
                        });
                    } else if let Some(ann) = find(&anns, "FormParam") {
                        form.push((named(ann), J::Str(default)));
                    } else if anns.is_empty() || has(&anns, "Valid") || has(&anns, "NotNull") {
                        let body = unwrap(ty);
                        declared.body_type = Some(last_segment(&generic(&body).0).to_string());
                        declared.body =
                            Some(sample(&body, "", &[], self.world, 0, &mut Vec::new()));
                        validated |= has(&anns, "Valid");
                        body_ty = Some(body);
                    }
                    continue;
                }
                if let Some(ann) = find(&anns, "RequestParam") {
                    if ty.contains("MultipartFile") {
                        multipart = true;
                        form.push((named(ann), J::Str(String::new())));
                    } else if !generic(ty).0.ends_with("Map") {
                        let needed = ann
                            .first(&["required"])
                            .map(|r| r != "false")
                            .unwrap_or(true)
                            && ann.first(&["defaultValue"]).is_none();
                        required |= needed;
                        declared.query.push(Param {
                            name: named(ann),
                            sample: ann.first(&["defaultValue"]).unwrap_or_default(),
                            required: needed,
                        });
                    }
                } else if let Some(ann) = find(&anns, "RequestHeader") {
                    declared.headers.push(Param {
                        name: named(ann),
                        sample: ann.first(&["defaultValue"]).unwrap_or_default(),
                        required: true,
                    });
                } else if let Some(ann) = find(&anns, "RequestPart") {
                    multipart = true;
                    form.push((named(ann), J::Str(String::new())));
                } else if has(&anns, "RequestBody") {
                    let body = unwrap(ty);
                    declared.body_type = Some(last_segment(&generic(&body).0).to_string());
                    declared.body = Some(sample(&body, "", &[], self.world, 0, &mut Vec::new()));
                    validated |= has(&anns, "Valid") || has(&anns, "Validated");
                    body_ty = Some(body);
                }
            }
        }
        if declared.body.is_none() && !form.is_empty() {
            declared.body = Some(J::Obj(form));
            if declared.consumes.is_none() || multipart {
                declared.consumes = Some(if multipart {
                    "multipart/form-data".into()
                } else {
                    "application/x-www-form-urlencoded".into()
                });
            }
        }

        let mut hint: Vec<String> = Vec::new();
        if let Some(body) = &declared.body_type {
            hint.push(body.clone());
        }
        let names: Vec<&str> = declared.query.iter().map(|p| p.name.as_str()).collect();
        if !names.is_empty() {
            hint.push(names.join(", "));
        }
        if let Some(c) = declared
            .consumes
            .as_ref()
            .filter(|c| c.as_str() != "application/json")
        {
            hint.push(c.clone());
        }
        declared.rules = body_ty
            .as_deref()
            .map(|ty| rules_of(ty, self.world))
            .unwrap_or_default();
        declared.responses = self.answers(
            source,
            class,
            class_anns,
            method,
            &path,
            validated && !declared.rules.is_empty(),
            required,
            jaxrs,
        );
        let call = build(&declared, &words_of(&name), Choices::default());
        self.out.push(Endpoint {
            id: format!("{}#{}", file.path, self.out.len()),
            key: key_of(&call),
            source: format!("code:{}", file.path),
            group: class.to_string(),
            file: Some(file.path.clone()),
            line: Some(method.start_position().row as u32 + 1),
            method: verb,
            path,
            hint: hint.join(" · "),
            call,
            status: Status::New,
            declared: Some(declared),
            folder: None,
        });
    }
}

/// A sample value as XML: an object's fields as child elements, a list as
/// its first item, `prefix` on every element (empty for unqualified ones).
fn xml_of(name: &str, value: &J, prefix: &str, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let tag = if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}:{name}")
    };
    match value {
        J::Obj(entries) if !entries.is_empty() => {
            out.push_str(&format!("{pad}<{tag}>\n"));
            for (key, item) in entries {
                xml_of(key, item, prefix, depth + 1, out);
            }
            out.push_str(&format!("{pad}</{tag}>\n"));
        }
        J::Arr(items) if !items.is_empty() => xml_of(name, &items[0], prefix, depth, out),
        J::Obj(_) | J::Arr(_) | J::Null => out.push_str(&format!("{pad}<{tag}/>\n")),
        other => out.push_str(&format!(
            "{pad}<{tag}>{}</{tag}>\n",
            escape(&other.text().unwrap_or_default())
        )),
    }
}

/// A package as JAX-WS's default namespace: `shop.billing` is
/// `http://billing.shop/`.
fn namespace_of(source: &str) -> String {
    let package = source
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("package "))
        .map(|p| p.trim_end_matches(';').trim())
        .unwrap_or("");
    let mut parts: Vec<&str> = package.split('.').filter(|p| !p.is_empty()).collect();
    parts.reverse();
    format!("http://{}/", parts.join("."))
}

impl Endpoints<'_> {
    fn soap_call(
        &self,
        service: &str,
        operation: &str,
        action: &str,
        url: &str,
        body: &str,
        ns: &str,
    ) -> Call {
        let ws_security = (self.soap.ws_security == "username").then(|| WsSecurity {
            mode: WsMode::Username,
            user: Some("{{user}}".into()),
            password: Some("{{password}}".into()),
            digest: false,
        });
        Call {
            id: new_id(),
            name: operation.to_string(),
            kind: Kind::Soap,
            url: url.to_string(),
            service: Some(service.to_string()),
            operation: Some(operation.to_string()),
            version: Some(self.soap.version.as_str().into()),
            action: (!action.is_empty()).then(|| action.to_string()),
            envelope: Some(envelope(
                self.soap.version,
                body,
                &[(ns.to_string(), "tns".to_string())],
            )),
            ws_security,
            ..Call::default()
        }
    }

    fn push_soap(&mut self, file: &Parsed, service: &str, call: Call, line: u32, hint: &str) {
        let operation = call.operation.clone().unwrap_or_default();
        self.out.push(Endpoint {
            id: format!("{}#{}", file.path, self.out.len()),
            key: key_of(&call),
            source: format!("code:{}", file.path),
            group: service.to_string(),
            file: Some(file.path.clone()),
            line: Some(line),
            method: "SOAP".into(),
            path: format!("{service} › {operation}"),
            hint: hint.to_string(),
            call,
            status: Status::New,
            declared: None,
            folder: None,
        });
    }

    /// JAX-WS: a `@WebService` class's public methods, or an interface's,
    /// as document/literal wrapped operations.
    fn jaxws(
        &mut self,
        file: &Parsed,
        class: &str,
        anns: &[Ann],
        methods: &[Node],
        method_anns: &[Vec<Ann>],
        interface: bool,
    ) {
        let source = &file.text;
        let web = find(anns, "WebService").cloned().unwrap_or_default();
        let service = web
            .first(&["serviceName"])
            .unwrap_or_else(|| format!("{class}Service"));
        let ns = web
            .first(&["targetNamespace"])
            .unwrap_or_else(|| namespace_of(source));
        for (method, anns) in methods.iter().zip(method_anns) {
            let public = interface
                || children(*method).into_iter().any(|c| {
                    c.kind() == "modifiers"
                        && text(c, source).split_whitespace().any(|w| w == "public")
                });
            let marked = find(anns, "WebMethod");
            if !public || marked.and_then(|m| m.first(&["exclude"])).as_deref() == Some("true") {
                continue;
            }
            let name = method
                .child_by_field_name("name")
                .map(|n| text(n, source).to_string())
                .unwrap_or_default();
            let operation = marked
                .and_then(|m| m.first(&["operationName"]))
                .unwrap_or(name);
            let action = marked
                .and_then(|m| m.first(&["action"]))
                .unwrap_or_default();
            let mut body = format!("    <tns:{operation}>\n");
            let params = method
                .child_by_field_name("parameters")
                .map(children)
                .unwrap_or_default();
            for (i, param) in params
                .into_iter()
                .filter(|p| p.kind() == "formal_parameter")
                .enumerate()
            {
                let pann = annotations(param, source, self.world, class);
                let pname = find(&pann, "WebParam")
                    .and_then(|w| w.first(&["name"]))
                    .unwrap_or_else(|| format!("arg{i}"));
                let ty = param
                    .child_by_field_name("type")
                    .map(|t| text(t, source))
                    .unwrap_or("String");
                let value = sample(ty, &pname, &[], self.world, 0, &mut Vec::new());
                xml_of(&pname, &value, "", 3, &mut body);
            }
            body.push_str(&format!("    </tns:{operation}>\n"));
            let url = format!("{{{{baseUrl}}}}/{service}");
            let call = self.soap_call(&service, &operation, &action, &url, &body, &ns);
            self.push_soap(
                file,
                &service,
                call,
                method.start_position().row as u32 + 1,
                "JAX-WS",
            );
        }
    }

    /// Spring-WS: each `@PayloadRoot` method of an `@Endpoint`, its
    /// `@RequestPayload` class as the payload.
    fn spring_ws(
        &mut self,
        file: &Parsed,
        class: &str,
        methods: &[Node],
        method_anns: &[Vec<Ann>],
    ) {
        let source = &file.text;
        for (method, anns) in methods.iter().zip(method_anns) {
            let Some(root) = find(anns, "PayloadRoot") else {
                continue;
            };
            let ns = root.first(&["namespace"]).unwrap_or_default();
            let Some(local) = root.first(&["localPart"]) else {
                continue;
            };
            let mut value = J::Obj(Vec::new());
            for param in method
                .child_by_field_name("parameters")
                .map(children)
                .unwrap_or_default()
            {
                if param.kind() == "formal_parameter"
                    && has(
                        &annotations(param, source, self.world, class),
                        "RequestPayload",
                    )
                {
                    let ty = param
                        .child_by_field_name("type")
                        .map(|t| text(t, source))
                        .unwrap_or("Object");
                    value = sample(ty, "", &[], self.world, 0, &mut Vec::new());
                }
            }
            let mut body = String::new();
            xml_of(&local, &value, "tns", 2, &mut body);
            let service = class.trim_end_matches("Endpoint").to_string();
            let service = if service.is_empty() {
                class.to_string()
            } else {
                service
            };
            let call = self.soap_call(&service, &local, "", "{{baseUrl}}/ws", &body, &ns);
            self.push_soap(
                file,
                &service,
                call,
                method.start_position().row as u32 + 1,
                "Spring-WS",
            );
        }
    }
}

/// Every endpoint the Java files declare, for the frameworks named
/// (`springMvc`, `springWebflux`, `jaxRs`, `jaxWs`, `springWs`), with a
/// source for each file that has any.
pub fn read(
    files: &[(String, String)],
    frameworks: &[String],
    soap: &SoapChoices,
) -> (Vec<Endpoint>, Vec<Source>) {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
        .is_err()
    {
        return (Vec::new(), Vec::new());
    }
    let parsed: Vec<Parsed> = files
        .iter()
        .filter_map(|(path, text)| {
            parser.parse(text, None).map(|tree| Parsed {
                path: path.clone(),
                text: text.clone(),
                tree,
            })
        })
        .collect();
    let world = index(&parsed);
    let mut endpoints = Endpoints {
        world: &world,
        frameworks,
        soap,
        out: Vec::new(),
        sources: Vec::new(),
    };
    for file in &parsed {
        endpoints.file(file);
    }
    (endpoints.out, endpoints.sources)
}

/// Whether a Java file is worth parsing: it mentions a mapping, a JAX-RS
/// annotation, or declares a type a body could be.
pub fn interesting(text: &str) -> bool {
    text.contains("class ")
        || text.contains("record ")
        || text.contains("enum ")
        || text.contains("interface ")
}

/// `${PORT:8081}` as `8081`; a plain value as it is.
fn placeholder(value: &str) -> String {
    let value = value.trim().trim_matches('"').trim_matches('\'');
    if let Some(inner) = value.strip_prefix("${").and_then(|v| v.strip_suffix('}')) {
        return inner
            .split_once(':')
            .map(|(_, d)| d.to_string())
            .unwrap_or_default();
    }
    value.to_string()
}

/// `http://localhost:<port><context>` from a Spring Boot `application.yml`
/// or `application.properties` (§8.2), the default profile's.
pub fn base_url(files: &[(String, String)]) -> Option<String> {
    let mut candidates: Vec<&(String, String)> = files
        .iter()
        .filter(|(path, _)| {
            let name = path.rsplit('/').next().unwrap_or(path);
            matches!(
                name,
                "application.yml" | "application.yaml" | "application.properties"
            )
        })
        .collect();
    candidates.sort_by_key(|(path, _)| (path.matches('/').count(), path.clone()));
    let (path, text) = candidates.first()?;
    let mut settings: HashMap<String, String> = HashMap::new();
    if path.ends_with(".properties") {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            if let Some((key, value)) = line.split_once(['=', ':']) {
                settings.insert(key.trim().to_string(), placeholder(value));
            }
        }
    } else {
        let docs = yaml_rust2::YamlLoader::load_from_str(text).ok()?;
        let doc = docs
            .iter()
            .find(|doc| {
                doc["spring"]["config"]["activate"]["on-profile"].is_badvalue()
                    && doc["spring"]["profiles"].is_badvalue()
            })
            .or(docs.first())?;
        let get = |parts: &[&str]| {
            let mut at = doc;
            for part in parts {
                at = &at[*part];
            }
            match at {
                yaml_rust2::Yaml::String(s) => Some(placeholder(s)),
                yaml_rust2::Yaml::Integer(i) => Some(i.to_string()),
                _ => None,
            }
        };
        for (key, parts) in [
            ("server.port", &["server", "port"][..]),
            (
                "server.servlet.context-path",
                &["server", "servlet", "context-path"][..],
            ),
            ("server.context-path", &["server", "context-path"][..]),
            (
                "spring.webflux.base-path",
                &["spring", "webflux", "base-path"][..],
            ),
        ] {
            if let Some(value) = get(parts) {
                settings.insert(key.to_string(), value);
            }
        }
    }
    let port = settings
        .get("server.port")
        .filter(|p| !p.is_empty() && *p != "0")
        .cloned()
        .unwrap_or_else(|| "8080".into());
    let context = [
        "server.servlet.context-path",
        "spring.webflux.base-path",
        "server.context-path",
    ]
    .iter()
    .find_map(|key| settings.get(*key).filter(|v| !v.is_empty()).cloned())
    .unwrap_or_default();
    let context = context.trim_end_matches('/');
    let context = if context.is_empty() || context.starts_with('/') {
        context.to_string()
    } else {
        format!("/{context}")
    };
    Some(format!("http://localhost:{port}{context}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(p, t)| (p.to_string(), t.to_string()))
            .collect()
    }

    const ALL: [&str; 3] = ["springMvc", "springWebflux", "jaxRs"];

    fn all() -> Vec<String> {
        ALL.map(String::from).to_vec()
    }

    const ORDERS: &str = r#"
package shop;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping(Paths.ORDERS)
public class OrderController {

    @GetMapping("/{id}")
    public Order get(@PathVariable("id") String id, @RequestHeader("X-Tenant") String tenant) { return null; }

    @GetMapping
    public List<Order> list(@RequestParam(defaultValue = "1") int page, @RequestParam(name = "q", required = false) String query) { return null; }

    @PostMapping(consumes = "application/json")
    public ResponseEntity<Order> create(@Valid @RequestBody CreateOrderRequest request) { return null; }

    @RequestMapping(value = "/{id}/cancel", method = RequestMethod.POST)
    public void cancel(@PathVariable String id) {}

    @PostMapping(value = "/{id}/receipt", consumes = "multipart/form-data")
    public void receipt(@PathVariable String id, @RequestParam("file") MultipartFile file) {}

    private void helper() {}
}
"#;

    const MODEL: &str = r#"
package shop;

public final class Paths {
    public static final String API = "/api";
    public static final String ORDERS = API + "/orders";
}

public class Base {
    private UUID id;
}

public class CreateOrderRequest extends Base {
    @NotNull @Size(min = 2) private String sku;
    @Min(1) private int quantity;
    @Email private String contact;
    private List<Line> lines;
    private Status status;
    @JsonProperty("coupon_code") private String couponCode;
    @JsonIgnore private String internal;
    private static final long serialVersionUID = 1L;
    private Map<String, BigDecimal> prices;
    private LocalDate due;
    private CreateOrderRequest parent;
}

public record Line(String sku, @Positive Integer qty, Money price) {}
public record Money(BigDecimal amount, String currency) {}
public enum Status { NEW, PAID }
"#;

    const PROMISES: &str = r#"
package shop;

@RestController
@RequestMapping("/accounts")
@PreAuthorize("hasRole('ADMIN')")
public class AccountController {
    private final AccountService accounts;

    @PostMapping
    @ApiResponse(responseCode = "409", description = "That email has an account already", content = @Content(schema = @Schema(implementation = Problem.class)))
    public ResponseEntity<Account> open(@Valid @RequestBody OpenAccount request) {
        Account made = accounts.open(request);
        return ResponseEntity.created(URI.create("/accounts/" + made.id())).body(made);
    }

    @GetMapping("/{id}")
    public Account get(@PathVariable Long id) {
        return accounts.find(id);
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void close(@PathVariable Long id) {
        if (id < 0) throw new ResponseStatusException(HttpStatus.GONE, "Closed already");
        accounts.close(id);
    }
}

public interface AccountService {
    Account open(OpenAccount request);
    Account find(Long id);
    void close(Long id);
}

@Service
public class DefaultAccountService implements AccountService {
    public Account open(OpenAccount request) {
        if (exists(request.email())) throw new EmailTakenException("That email has an account already");
        return null;
    }
    public Account find(Long id) {
        return repository.findById(id).orElseThrow(() -> new AccountNotFoundException(id));
    }
    public void close(Long id) {}
}

@ResponseStatus(HttpStatus.NOT_FOUND)
public class AccountNotFoundException extends RuntimeException {}

public class EmailTakenException extends RuntimeException {}

@RestControllerAdvice
public class Errors {
    @ExceptionHandler(EmailTakenException.class)
    public ResponseEntity<Problem> taken(EmailTakenException e) {
        return ResponseEntity.status(HttpStatus.CONFLICT).body(new Problem(e.getMessage()));
    }
}

public record Problem(String message) {}

public record Account(Long id, String email) {}

public enum Plan { FREE, PRO }

public record OpenAccount(
    @NotBlank @Email String email,
    @Size(min = 3, max = 40) String name,
    @Min(18) @Max(130) int age,
    @NotNull Plan plan,
    @Pattern(regexp = "^[A-Z]{2}$") String country,
    @Valid List<Address> addresses
) {}

public record Address(@NotBlank String line, String city) {}
"#;

    /// FEAT-179: what an endpoint promises, from the code around it.
    #[test]
    fn a_body_says_its_rules_and_a_method_says_what_it_answers() {
        let (endpoints, _) = read(
            &files(&[("src/Accounts.java", PROMISES)]),
            &all(),
            &SoapChoices::default(),
        );
        let declared = |path: &str, method: &str| {
            endpoints
                .iter()
                .find(|e| e.path == path && e.method == method)
                .and_then(|e| e.declared.clone())
                .expect("the endpoint")
        };

        let open = declared("/accounts", "POST");
        let rule = |path: &str| {
            open.rules
                .iter()
                .find(|r| r.path == path)
                .cloned()
                .expect(path)
        };
        let email = rule("email");
        assert_eq!(
            (
                email.kind.as_str(),
                email.required,
                email.format.as_deref(),
                email.min_length
            ),
            ("string", true, Some("email"), Some(1))
        );
        let name = rule("name");
        assert_eq!(
            (name.min_length, name.max_length, name.required),
            (Some(3), Some(40), false)
        );
        let age = rule("age");
        assert_eq!(
            (age.kind.as_str(), age.minimum, age.maximum),
            ("integer", Some(18.0), Some(130.0))
        );
        assert_eq!(rule("plan").values, ["FREE", "PRO"]);
        assert_eq!(rule("country").pattern.as_deref(), Some("^[A-Z]{2}$"));
        assert_eq!(rule("addresses").kind, "array");
        assert!(rule("addresses[].line").required);

        let statuses = |responses: &[Answer]| {
            responses
                .iter()
                .map(|a| (a.status, a.from.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            statuses(&open.responses),
            [
                (201, "the return type".to_string()),
                (400, "@Valid".to_string()),
                (401, "@PreAuthorize".to_string()),
                (403, "@PreAuthorize".to_string()),
                (409, "@ApiResponse".to_string()),
                (500, "any endpoint".to_string())
            ]
        );
        let created = &open.responses[0];
        assert!(
            created.body.as_deref().unwrap_or("").contains("\"email\""),
            "{created:?}"
        );
        // The author's words win; the handler's body is kept as the example.
        let conflict = open
            .responses
            .iter()
            .find(|a| a.status == 409)
            .expect("409");
        assert_eq!(conflict.says, "That email has an account already");
        assert!(
            conflict
                .body
                .as_deref()
                .unwrap_or("")
                .contains("\"message\""),
            "{conflict:?}"
        );
        assert!(
            open.responses[2].says.contains("hasRole('ADMIN')")
                || open.responses[3].says.contains("hasRole('ADMIN')")
        );

        // A service's exception, one call deep, at its own @ResponseStatus.
        let get = declared("/accounts/{id}", "GET");
        let missing = get.responses.iter().find(|a| a.status == 404).expect("404");
        assert_eq!(
            (missing.says.as_str(), missing.from.as_str()),
            ("Account not found", "throws AccountNotFoundException")
        );
        assert!(
            missing
                .body
                .as_deref()
                .unwrap_or("")
                .contains("\"status\": 404"),
            "{missing:?}"
        );

        // @ResponseStatus on the method, and a status the body raises itself.
        let close = declared("/accounts/{id}", "DELETE");
        assert_eq!(
            (
                close.responses[0].status,
                close.responses[0].body.as_deref()
            ),
            (204, None)
        );
        let gone = close
            .responses
            .iter()
            .find(|a| a.status == 410)
            .expect("410");
        assert_eq!(gone.says, "Closed already");
    }

    #[test]
    fn spring_controllers_give_their_endpoints() {
        let (endpoints, sources) = read(
            &files(&[
                ("src/OrderController.java", ORDERS),
                ("src/Model.java", MODEL),
            ]),
            &all(),
            &SoapChoices::default(),
        );
        let got: Vec<(String, String, String)> = endpoints
            .iter()
            .map(|e| (e.method.clone(), e.path.clone(), e.call.url.clone()))
            .collect();
        assert_eq!(
            got,
            vec![
                (
                    "GET".into(),
                    "/api/orders/{id}".into(),
                    "{{baseUrl}}/api/orders/{{id}}".into()
                ),
                (
                    "GET".into(),
                    "/api/orders".into(),
                    "{{baseUrl}}/api/orders?page=1&q".into()
                ),
                (
                    "POST".into(),
                    "/api/orders".into(),
                    "{{baseUrl}}/api/orders".into()
                ),
                (
                    "POST".into(),
                    "/api/orders/{id}/cancel".into(),
                    "{{baseUrl}}/api/orders/{{id}}/cancel".into()
                ),
                (
                    "POST".into(),
                    "/api/orders/{id}/receipt".into(),
                    "{{baseUrl}}/api/orders/{{id}}/receipt".into()
                ),
            ]
        );
        assert_eq!(endpoints[0].call.headers[0].name, "X-Tenant");
        assert_eq!(endpoints[0].group, "OrderController");
        assert_eq!(endpoints[0].call.name, "Get");
        assert_eq!(endpoints[1].hint, "page, q");
        assert_eq!(endpoints[2].hint, "CreateOrderRequest");
        assert_eq!(
            endpoints[4].call.body.as_ref().unwrap().kind,
            crate::calls::model::BodyType::Multipart
        );
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].count, 5);
        assert_eq!(sources[0].label, "OrderController.java");
    }

    #[test]
    fn a_request_body_is_an_example_of_its_class() {
        let (endpoints, _) = read(
            &files(&[
                ("src/OrderController.java", ORDERS),
                ("src/Model.java", MODEL),
            ]),
            &all(),
            &SoapChoices::default(),
        );
        let body = endpoints[2]
            .call
            .body
            .as_ref()
            .unwrap()
            .text
            .clone()
            .unwrap();
        let expected = r#"{
  "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "sku": "sku",
  "quantity": 1,
  "contact": "mira@example.com",
  "lines": [
    {
      "sku": "sku",
      "qty": 1,
      "price": {
        "amount": 0.0,
        "currency": "currency"
      }
    }
  ],
  "status": "NEW",
  "coupon_code": "couponCode",
  "prices": {
    "key": 0.0
  },
  "due": "2026-01-31",
  "parent": null
}"#;
        assert_eq!(body, expected);
    }

    #[test]
    fn webflux_and_jaxrs_are_read_and_can_be_turned_off() {
        let flux = r#"
import reactor.core.publisher.Mono;
@RestController
@RequestMapping("/v2/users")
class UserHandler {
    @PutMapping("/{userId}")
    Mono<User> replace(@PathVariable String userId, @RequestBody Mono<User> user) { return null; }
}
record User(String name) {}
"#;
        let jaxrs = r#"
@ApplicationPath("api")
public class App extends Application {}

@Path("/invoices")
@Consumes(MediaType.APPLICATION_JSON)
public interface InvoiceResource {
    @GET @Path("{id: [0-9]+}")
    Invoice get(@PathParam("id") long id, @QueryParam("lang") @DefaultValue("en") String lang);

    @POST
    Response create(Invoice invoice);

    @POST @Path("/login") @Consumes(MediaType.APPLICATION_FORM_URLENCODED)
    Response login(@FormParam("user") String user, @FormParam("password") String password);
}
record Invoice(String number) {}
"#;
        let list = files(&[("a/UserHandler.java", flux), ("b/Invoices.java", jaxrs)]);
        let (endpoints, sources) = read(&list, &all(), &SoapChoices::default());
        let got: Vec<String> = endpoints
            .iter()
            .map(|e| format!("{} {}", e.method, e.call.url))
            .collect();
        assert_eq!(
            got,
            [
                "PUT {{baseUrl}}/v2/users/{{userId}}",
                "GET {{baseUrl}}/api/invoices/{{id}}?lang=en",
                "POST {{baseUrl}}/api/invoices",
                "POST {{baseUrl}}/api/invoices/login",
            ]
        );
        assert_eq!(
            endpoints[0].call.body.as_ref().unwrap().text.as_deref(),
            Some("{\n  \"name\": \"name\"\n}")
        );
        assert_eq!(endpoints[3].call.body.as_ref().unwrap().fields.len(), 2);
        assert_eq!(
            sources.iter().map(|s| s.kind).collect::<Vec<_>>(),
            [SourceKind::Spring, SourceKind::JaxRs]
        );

        let (only_mvc, _) = read(&list, &["springMvc".to_string()], &SoapChoices::default());
        assert!(only_mvc.is_empty());
    }

    #[test]
    fn jaxws_and_spring_ws_endpoints_are_soap_calls_with_envelopes() {
        let jaxws = r#"
package shop.billing;

@WebService(serviceName = "InvoiceService")
public class InvoiceEndpoint {
    @WebMethod(operationName = "GetInvoice", action = "urn:GetInvoice")
    public Invoice get(@WebParam(name = "invoiceId") String id, @WebParam(name = "query") Query q) { return null; }

    public void ping(int times) {}

    @WebMethod(exclude = true)
    public void hidden() {}

    private void helper() {}
}
record Query(String lang, int page) {}
"#;
        let spring_ws = r#"
@Endpoint
public class CountryEndpoint {
    private static final String NS = "http://spring.io/guides/gs-producing-web-service";

    @PayloadRoot(namespace = NS, localPart = "getCountryRequest")
    @ResponsePayload
    public GetCountryResponse getCountry(@RequestPayload GetCountryRequest request) { return null; }
}
class GetCountryRequest { private String name; }
"#;
        let frameworks = vec!["jaxWs".to_string(), "springWs".to_string()];
        let (endpoints, sources) = read(
            &files(&[
                ("src/InvoiceEndpoint.java", jaxws),
                ("src/CountryEndpoint.java", spring_ws),
            ]),
            &frameworks,
            &SoapChoices::default(),
        );
        let got: Vec<(String, String, String)> = endpoints
            .iter()
            .map(|e| (e.key.clone(), e.call.url.clone(), e.hint.clone()))
            .collect();
        assert_eq!(
            got,
            [
                (
                    "SOAP InvoiceService#GetInvoice".into(),
                    "{{baseUrl}}/InvoiceService".into(),
                    "JAX-WS".into()
                ),
                (
                    "SOAP InvoiceService#ping".into(),
                    "{{baseUrl}}/InvoiceService".into(),
                    "JAX-WS".into()
                ),
                (
                    "SOAP Country#getCountryRequest".into(),
                    "{{baseUrl}}/ws".into(),
                    "Spring-WS".into()
                ),
            ]
        );
        let get = &endpoints[0].call;
        assert_eq!(get.action.as_deref(), Some("urn:GetInvoice"));
        let envelope = get.envelope.as_deref().unwrap();
        assert!(envelope.contains("xmlns:tns=\"http://billing.shop/\""));
        assert!(envelope.contains("    <tns:GetInvoice>\n      <invoiceId>invoiceId</invoiceId>\n      <query>\n        <lang>lang</lang>\n        <page>0</page>\n      </query>\n    </tns:GetInvoice>"));
        assert!(endpoints[1]
            .call
            .envelope
            .as_deref()
            .unwrap()
            .contains("<arg0>0</arg0>"));
        let country = endpoints[2].call.envelope.as_deref().unwrap();
        assert!(country.contains("xmlns:tns=\"http://spring.io/guides/gs-producing-web-service\""));
        assert!(country.contains("    <tns:getCountryRequest>\n      <tns:name>name</tns:name>\n    </tns:getCountryRequest>"));
        assert_eq!(
            sources.iter().map(|s| s.kind).collect::<Vec<_>>(),
            [SourceKind::JaxWs, SourceKind::SpringWs]
        );
    }

    #[test]
    fn the_server_address_comes_from_application_yml() {
        let yml = "spring:\n  application:\n    name: orders\nserver:\n  port: ${PORT:8081}\n  servlet:\n    context-path: /shop\n---\nspring:\n  config:\n    activate:\n      on-profile: prod\nserver:\n  port: 80\n";
        assert_eq!(
            base_url(&files(&[("svc/src/main/resources/application.yml", yml)])).as_deref(),
            Some("http://localhost:8081/shop")
        );
        assert_eq!(
            base_url(&files(&[(
                "src/main/resources/application.properties",
                "server.port = 9000\n"
            )]))
            .as_deref(),
            Some("http://localhost:9000")
        );
        assert_eq!(
            base_url(&files(&[(
                "src/main/resources/application.yml",
                "spring:\n  webflux:\n    base-path: api\n"
            )]))
            .as_deref(),
            Some("http://localhost:8080/api")
        );
        assert_eq!(base_url(&files(&[("pom.xml", "")])), None);
    }
}
