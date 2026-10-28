// SPDX-License-Identifier: MIT

//! Endpoints declared by NestJS controllers (FEAT-162; the author asked for
//! NestJS on 2026-10-26).
//!
//! Read from TypeScript with the tree-sitter grammar Files already compiles
//! in, the way the Java reader reads Spring: `@Controller('orders')` on the
//! class, `@Get(':id')` and the others on its methods, and `@Param`,
//! `@Query`, `@Headers` and `@Body` on their parameters. A body's example
//! comes from its DTO class or interface, shaped by class-validator
//! (`@IsEmail`, `@Min(1)`, `@IsEnum(Status)`) and by Swagger's
//! `@ApiProperty({ example })` where it is given. `setGlobalPrefix('api')`
//! in `main.ts` is the paths' prefix, and the port `listen` is given is the
//! `baseUrl`.
//!
//! # What an endpoint promises (FEAT-179)
//!
//! The DTO's class-validator decorators are its fields' rules, and a 400 in
//! the shape `ValidationPipe` sends, its messages written from them. The
//! method's `@HttpCode` or its verb is its success (Nest answers a POST
//! with 201), its return type the example; `@UseGuards` is a 401 and a 403;
//! each `HttpException` it throws — or the service it calls throws, one
//! call deep — at Nest's status for it; `@nestjs/swagger`'s
//! `@ApiResponse`s are the author's own word.

use std::collections::HashMap;

use tree_sitter::{Node, Parser};

use super::super::model::{Answer, FieldRule};
use super::contract::{self, answer, Answers, DECLARED, INFERRED, THROWN};
use super::json::J;
use super::{
    build, key_of, words_of, Choices, Declared, Endpoint, Param, Source, SourceKind, Status,
};

const DEPTH: usize = 4;
const VERBS: [(&str, &str); 7] = [
    ("Get", "GET"),
    ("Post", "POST"),
    ("Put", "PUT"),
    ("Patch", "PATCH"),
    ("Delete", "DELETE"),
    ("Head", "HEAD"),
    ("Options", "OPTIONS"),
];

/// A decorator: its name and its arguments, as the text of each and, for
/// an object argument, its keys.
#[derive(Debug, Clone, Default)]
struct Deco {
    name: String,
    /// String arguments' contents; anything else as written.
    args: Vec<String>,
    /// `{ path: 'x', example: 3 }` as its entries.
    object: Vec<(String, J)>,
}

impl Deco {
    fn first(&self) -> Option<&str> {
        self.args.first().map(String::as_str)
    }
}

#[derive(Debug, Clone)]
struct Prop {
    name: String,
    ty: String,
    decos: Vec<Deco>,
    /// Written `name?: …`.
    optional: bool,
}

#[derive(Default)]
struct World {
    /// Classes and interfaces, by name.
    types: HashMap<String, Vec<Prop>>,
    /// Enums, by name, with their values.
    enums: HashMap<String, Vec<J>>,
    /// A class's parent, by name: an exception's lineage (FEAT-179).
    parents: HashMap<String, String>,
    /// A class's methods, with their bodies as text.
    methods: HashMap<String, Vec<(String, String)>>,
    /// What a class's constructor is given: the services it holds.
    injected: HashMap<String, Vec<(String, String)>>,
}

/// Nest's own exceptions, and the status each sends.
const NEST_EXCEPTIONS: [(&str, u16); 16] = [
    ("BadRequestException", 400),
    ("UnauthorizedException", 401),
    ("ForbiddenException", 403),
    ("NotFoundException", 404),
    ("MethodNotAllowedException", 405),
    ("NotAcceptableException", 406),
    ("RequestTimeoutException", 408),
    ("ConflictException", 409),
    ("GoneException", 410),
    ("PayloadTooLargeException", 413),
    ("UnsupportedMediaTypeException", 415),
    ("UnprocessableEntityException", 422),
    ("InternalServerErrorException", 500),
    ("NotImplementedException", 501),
    ("BadGatewayException", 502),
    ("ServiceUnavailableException", 503),
];

/// `@nestjs/swagger`'s response decorators, and the status each is for.
const SWAGGER_RESPONSES: [(&str, u16); 12] = [
    ("ApiOkResponse", 200),
    ("ApiCreatedResponse", 201),
    ("ApiAcceptedResponse", 202),
    ("ApiNoContentResponse", 204),
    ("ApiBadRequestResponse", 400),
    ("ApiUnauthorizedResponse", 401),
    ("ApiForbiddenResponse", 403),
    ("ApiNotFoundResponse", 404),
    ("ApiConflictResponse", 409),
    ("ApiUnprocessableEntityResponse", 422),
    ("ApiTooManyRequestsResponse", 429),
    ("ApiInternalServerErrorResponse", 500),
];

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

fn children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

/// A literal as a value: strings without quotes, numbers and booleans as
/// they are, objects and arrays inside out.
fn literal(node: Node, source: &str) -> J {
    match node.kind() {
        "string" | "template_string" => J::Str(
            text(node, source)
                .trim_matches(|c| c == '\'' || c == '"' || c == '`')
                .to_string(),
        ),
        "number" => J::Num(text(node, source).to_string()),
        "true" => J::Bool(true),
        "false" => J::Bool(false),
        "null" | "undefined" => J::Null,
        "array" => J::Arr(
            children(node)
                .into_iter()
                .map(|c| literal(c, source))
                .collect(),
        ),
        "object" => J::Obj(
            children(node)
                .into_iter()
                .filter(|c| c.kind() == "pair")
                .filter_map(|pair| {
                    let key = pair.child_by_field_name("key")?;
                    let value = pair.child_by_field_name("value")?;
                    Some((
                        text(key, source).trim_matches(['\'', '"']).to_string(),
                        literal(value, source),
                    ))
                })
                .collect(),
        ),
        _ => J::Str(text(node, source).to_string()),
    }
}

fn decorator(node: Node, source: &str) -> Option<Deco> {
    let inner = children(node).into_iter().next()?;
    let (callee, args) = match inner.kind() {
        "call_expression" => (
            inner.child_by_field_name("function")?,
            inner.child_by_field_name("arguments"),
        ),
        _ => (inner, None),
    };
    let name = text(callee, source)
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_string();
    let mut deco = Deco {
        name,
        ..Deco::default()
    };
    for arg in args.map(children).unwrap_or_default() {
        match literal(arg, source) {
            J::Obj(entries) => deco.object = entries,
            J::Str(s) => deco.args.push(s),
            other => deco.args.push(other.text().unwrap_or_default()),
        }
    }
    Some(deco)
}

/// The decorators that are a node's own children.
fn own_decorators(node: Node, source: &str) -> Vec<Deco> {
    children(node)
        .into_iter()
        .filter(|c| c.kind() == "decorator")
        .filter_map(|c| decorator(c, source))
        .collect()
}

fn find<'a>(decos: &'a [Deco], name: &str) -> Option<&'a Deco> {
    decos.iter().find(|d| d.name == name)
}

/// A type annotation's type, as written: `: Order[]` is `Order[]`.
fn type_text(node: Option<Node>, source: &str) -> String {
    node.map(|n| text(n, source).trim_start_matches(':').trim().to_string())
        .unwrap_or_default()
}

/// `{ path: 'x' }` or `'x'`: the path a decorator gives.
fn path_arg(deco: &Deco) -> String {
    deco.first()
        .map(str::to_string)
        .or_else(|| {
            deco.object
                .iter()
                .find(|(k, _)| k == "path")
                .and_then(|(_, v)| v.text())
        })
        .unwrap_or_default()
}

/// `orders/:id` as `orders/{id}`.
fn braces(path: &str) -> String {
    path.split('/')
        .map(|s| match s.strip_prefix(':') {
            Some(name) => format!("{{{}}}", name.trim_end_matches('?')),
            None => s.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn join(parts: &[&str]) -> String {
    let joined: Vec<&str> = parts
        .iter()
        .map(|p| p.trim().trim_matches('/'))
        .filter(|p| !p.is_empty())
        .collect();
    format!("/{}", joined.join("/"))
}

/// A sample of the TypeScript type `ty`.
fn sample(
    ty: &str,
    field: &str,
    decos: &[Deco],
    world: &World,
    depth: usize,
    seen: &mut Vec<String>,
) -> J {
    if let Some(example) = find(decos, "ApiProperty")
        .or_else(|| find(decos, "ApiPropertyOptional"))
        .and_then(|d| d.object.iter().find(|(k, _)| k == "example"))
    {
        return example.1.clone();
    }
    let ty = ty.trim();
    // `string | null`, `'a' | 'b'`: the first that is not nothing.
    if ty.contains('|') {
        let first = ty
            .split('|')
            .map(str::trim)
            .find(|t| !matches!(*t, "null" | "undefined"))
            .unwrap_or("null");
        if first.starts_with(['\'', '"']) {
            return J::Str(first.trim_matches(['\'', '"']).to_string());
        }
        return sample(first, field, decos, world, depth, seen);
    }
    if let Some(element) = ty.strip_suffix("[]") {
        return J::Arr(vec![sample(element, field, decos, world, depth, seen)]);
    }
    let (base, args) = match ty.find('<') {
        Some(at) => (
            &ty[..at],
            ty[at + 1..ty.rfind('>').unwrap_or(ty.len())]
                .split(',')
                .map(str::trim)
                .collect::<Vec<_>>(),
        ),
        None => (ty, Vec::new()),
    };
    let arg = |at: usize| args.get(at).copied().unwrap_or("any");
    let has = |name: &str| decos.iter().any(|d| d.name == name);
    let min = |name: &str| {
        find(decos, name)
            .and_then(|d| d.first())
            .map(str::to_string)
    };
    if let Some(enum_name) = find(decos, "IsEnum").and_then(|d| d.first()) {
        if let Some(first) = world.enums.get(enum_name).and_then(|v| v.first()) {
            return first.clone();
        }
    }
    if let Some(nested) = find(decos, "Type").and_then(|d| d.first()) {
        // `@Type(() => Line)`.
        let name = nested.rsplit("=>").next().unwrap_or(nested).trim();
        if world.types.contains_key(name) && base != name {
            let inner = sample(name, field, &[], world, depth, seen);
            return if base == "Array" || ty.ends_with("[]") {
                J::Arr(vec![inner])
            } else {
                inner
            };
        }
    }
    match base {
        "string" | "String" => {
            let lower = field.to_ascii_lowercase();
            J::Str(if has("IsEmail") || lower.contains("email") {
                "mira@example.com".into()
            } else if has("IsUUID") {
                "3fa85f64-5717-4562-b3fc-2c963f66afa6".into()
            } else if has("IsDateString") || has("IsISO8601") {
                "2026-01-31T09:30:00Z".into()
            } else if has("IsUrl") || lower.ends_with("url") {
                "https://example.com".into()
            } else if let Some(n) = min("MinLength")
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|n| *n > field.len())
            {
                format!("{field}{}", "x".repeat(n - field.len()))
            } else if field.is_empty() {
                "string".into()
            } else {
                field.to_string()
            })
        }
        "number" | "Number" | "bigint" => J::Num(
            min("Min")
                .or_else(|| has("IsPositive").then(|| "1".to_string()))
                .unwrap_or_else(|| "0".into()),
        ),
        "boolean" | "Boolean" => J::Bool(false),
        "Date" => J::Str("2026-01-31T09:30:00Z".into()),
        "Array" | "ReadonlyArray" | "Set" => {
            J::Arr(vec![sample(arg(0), field, &[], world, depth, seen)])
        }
        "Record" | "Map" => J::Obj(vec![(
            "key".into(),
            sample(arg(1), "", &[], world, depth, seen),
        )]),
        "Promise" | "Observable" | "Partial" | "Required" | "Readonly" => {
            sample(arg(0), field, decos, world, depth, seen)
        }
        "any" | "unknown" | "object" | "Object" => J::Obj(Vec::new()),
        name => {
            if let Some(first) = world.enums.get(name).and_then(|v| v.first()) {
                return first.clone();
            }
            match world.types.get(name) {
                Some(_) if depth >= DEPTH || seen.iter().any(|s| s == name) => J::Null,
                Some(props) => {
                    seen.push(name.to_string());
                    let out = J::Obj(
                        props
                            .iter()
                            .map(|p| {
                                (
                                    p.name.clone(),
                                    sample(&p.ty, &p.name, &p.decos, world, depth + 1, seen),
                                )
                            })
                            .collect(),
                    );
                    seen.pop();
                    out
                }
                None => J::Null,
            }
        }
    }
}

struct Parsed {
    path: String,
    text: String,
    tree: tree_sitter::Tree,
}

/// Top-level declarations, each with the decorators written before it
/// (an exported class keeps them on its `export` statement).
fn declarations<'t>(root: Node<'t>, source: &str) -> Vec<(Node<'t>, Vec<Deco>)> {
    let mut out = Vec::new();
    for child in children(root) {
        match child.kind() {
            "export_statement" => {
                let decos = own_decorators(child, source);
                if let Some(decl) = child.child_by_field_name("declaration") {
                    let mut all = decos;
                    all.extend(own_decorators(decl, source));
                    out.push((decl, all));
                }
            }
            _ => {
                let decos = own_decorators(child, source);
                out.push((child, decos));
            }
        }
    }
    out
}

fn index(files: &[Parsed]) -> World {
    let mut world = World::default();
    for file in files {
        let source = &file.text;
        for (decl, _) in declarations(file.tree.root_node(), source) {
            let Some(name) = decl
                .child_by_field_name("name")
                .map(|n| text(n, source).to_string())
            else {
                continue;
            };
            let Some(body) = decl.child_by_field_name("body") else {
                continue;
            };
            match decl.kind() {
                "class_declaration" | "abstract_class_declaration" => {
                    let mut pending: Vec<Deco> = Vec::new();
                    let mut props = Vec::new();
                    for member in children(body) {
                        match member.kind() {
                            "decorator" => pending.extend(decorator(member, source)),
                            "public_field_definition" => {
                                let mut decos = std::mem::take(&mut pending);
                                decos.extend(own_decorators(member, source));
                                if text(member, source).trim_start().starts_with("static")
                                    || decos
                                        .iter()
                                        .any(|d| d.name == "Exclude" || d.name == "ApiHideProperty")
                                {
                                    continue;
                                }
                                if let Some(field) = member.child_by_field_name("name") {
                                    props.push(Prop {
                                        name: text(field, source).to_string(),
                                        ty: type_text(member.child_by_field_name("type"), source),
                                        decos,
                                        optional: text(member, source).contains("?:"),
                                    });
                                }
                            }
                            "method_definition" => {
                                pending.clear();
                                let method = member
                                    .child_by_field_name("name")
                                    .map(|n| text(n, source).to_string())
                                    .unwrap_or_default();
                                if method == "constructor" {
                                    let held = member
                                        .child_by_field_name("parameters")
                                        .map(children)
                                        .unwrap_or_default()
                                        .into_iter()
                                        .filter_map(|param| {
                                            Some((
                                                text(param.child_by_field_name("pattern")?, source)
                                                    .to_string(),
                                                type_text(
                                                    param.child_by_field_name("type"),
                                                    source,
                                                ),
                                            ))
                                        })
                                        .collect();
                                    world.injected.insert(name.clone(), held);
                                }
                                let body = member
                                    .child_by_field_name("body")
                                    .map(|b| text(b, source).to_string())
                                    .unwrap_or_default();
                                world
                                    .methods
                                    .entry(name.clone())
                                    .or_default()
                                    .push((method, body));
                            }
                            _ => pending.clear(),
                        }
                    }
                    let heritage = children(decl)
                        .into_iter()
                        .find(|c| c.kind() == "class_heritage")
                        .map(|h| text(h, source).to_string())
                        .unwrap_or_default();
                    if let Some(parent) = heritage
                        .split_whitespace()
                        .skip_while(|w| *w != "extends")
                        .nth(1)
                    {
                        let parent = parent.split(['<', '{', ',']).next().unwrap_or(parent);
                        world.parents.insert(name.clone(), parent.to_string());
                    }
                    world.types.entry(name).or_insert(props);
                }
                "interface_declaration" => {
                    let props = children(body)
                        .into_iter()
                        .filter(|m| m.kind() == "property_signature")
                        .filter_map(|m| {
                            Some(Prop {
                                name: text(m.child_by_field_name("name")?, source).to_string(),
                                ty: type_text(m.child_by_field_name("type"), source),
                                decos: Vec::new(),
                                optional: text(m, source).contains("?:"),
                            })
                        })
                        .collect();
                    world.types.entry(name).or_insert(props);
                }
                "enum_declaration" => {
                    let values = children(body)
                        .into_iter()
                        .filter_map(|m| match m.kind() {
                            "enum_assignment" => {
                                m.child_by_field_name("value").map(|v| literal(v, source))
                            }
                            "property_identifier" => Some(J::Str(text(m, source).to_string())),
                            _ => None,
                        })
                        .collect();
                    world.enums.insert(name, values);
                }
                _ => {}
            }
        }
    }
    world
}

/// The rules a DTO's fields are held to, a field per path (FEAT-179).
fn rules_of(ty: &str, world: &World) -> Vec<FieldRule> {
    let mut out = Vec::new();
    walk_rules(ty, "", world, 0, &mut Vec::new(), &mut out);
    out
}

/// `Order[]`, `Array<Order>` and `Promise<Order>` as `Order`, and whether it was a list.
fn element(ty: &str) -> (String, bool) {
    let ty = ty.trim();
    if let Some(inner) = ty.strip_suffix("[]") {
        return (element(inner).0, true);
    }
    if let Some(at) = ty.find('<') {
        let base = &ty[..at];
        let inner = ty[at + 1..ty.rfind('>').unwrap_or(ty.len())].trim();
        match base {
            "Array" | "ReadonlyArray" | "Set" => return (element(inner).0, true),
            "Promise" | "Observable" | "Partial" | "Required" | "Readonly" => {
                return element(inner)
            }
            _ => {}
        }
    }
    (
        ty.split('|')
            .map(str::trim)
            .find(|t| !matches!(*t, "null" | "undefined"))
            .unwrap_or(ty)
            .to_string(),
        false,
    )
}

fn walk_rules(
    ty: &str,
    prefix: &str,
    world: &World,
    depth: usize,
    seen: &mut Vec<String>,
    out: &mut Vec<FieldRule>,
) {
    let (name, list) = element(ty);
    let prefix = if list {
        format!("{prefix}[]")
    } else {
        prefix.to_string()
    };
    let Some(props) = world.types.get(&name) else {
        return;
    };
    if depth >= DEPTH || seen.contains(&name) {
        return;
    }
    seen.push(name.clone());
    for prop in props {
        let path = if prefix.is_empty() {
            prop.name.clone()
        } else {
            format!("{prefix}.{}", prop.name)
        };
        out.push(rule_of(&path, prop, world));
        let nested = find(&prop.decos, "Type")
            .and_then(|d| d.first())
            .map(|t| t.rsplit("=>").next().unwrap_or(t).trim().to_string());
        let (inner, many) = element(&prop.ty);
        let target = nested.unwrap_or(inner);
        let target = if many { format!("{target}[]") } else { target };
        walk_rules(&target, &path, world, depth + 1, seen, out);
    }
    seen.pop();
}

/// A number a decorator was given.
fn bound(deco: &Deco, at: usize) -> Option<f64> {
    deco.args.get(at).and_then(|v| v.trim().parse::<f64>().ok())
}

/// One DTO field's rule, from its type and class-validator's decorators.
fn rule_of(path: &str, prop: &Prop, world: &World) -> FieldRule {
    let has = |name: &str| prop.decos.iter().any(|d| d.name == name);
    let (inner, list) = element(&prop.ty);
    let enum_values = |name: &str| -> Vec<String> {
        world
            .enums
            .get(name)
            .map(|values| values.iter().filter_map(J::text).collect())
            .unwrap_or_default()
    };
    let (kind, mut format, mut values) = if list {
        ("array", None, Vec::new())
    } else {
        match inner.as_str() {
            "string" | "String" => ("string", None, Vec::new()),
            "number" | "Number" | "bigint" => (
                if has("IsInt") { "integer" } else { "number" },
                None,
                Vec::new(),
            ),
            "boolean" | "Boolean" => ("boolean", None, Vec::new()),
            "Date" => ("string", Some("date-time"), Vec::new()),
            name if world.enums.contains_key(name) => ("string", None, enum_values(name)),
            name if name.starts_with(['\'', '"']) => (
                "string",
                None,
                prop.ty
                    .split('|')
                    .map(|v| v.trim().trim_matches(['\'', '"']).to_string())
                    .collect(),
            ),
            _ => ("object", None, Vec::new()),
        }
    };
    if let Some(deco) = find(&prop.decos, "IsEnum") {
        values = deco.first().map(enum_values).unwrap_or_default();
    }
    if let Some(deco) = find(&prop.decos, "IsIn") {
        values = deco
            .args
            .iter()
            .map(|v| v.trim_matches(['[', ']', '\'', '"', ' ']).to_string())
            .collect();
    }
    if has("IsEmail") {
        format = Some("email");
    } else if has("IsUUID") {
        format = Some("uuid");
    } else if has("IsDateString") || has("IsISO8601") {
        format = Some("date-time");
    } else if has("IsUrl") {
        format = Some("uri");
    }
    let validated = prop.decos.iter().any(|d| {
        d.name.starts_with("Is")
            || matches!(
                d.name.as_str(),
                "Min" | "Max" | "Length" | "MinLength" | "MaxLength" | "Matches" | "ValidateNested"
            )
    });
    let mut rule = FieldRule {
        path: path.to_string(),
        kind: kind.to_string(),
        required: !prop.optional && !has("IsOptional") && (validated || has("IsDefined")),
        format: format.map(str::to_string),
        values,
        ..FieldRule::default()
    };
    if kind == "string" && has("IsNotEmpty") {
        rule.min_length = Some(1);
    }
    if let Some(deco) = find(&prop.decos, "Length") {
        rule.min_length = bound(deco, 0).map(|v| v as u64);
        rule.max_length = bound(deco, 1).map(|v| v as u64);
    }
    if let Some(deco) = find(&prop.decos, "MinLength") {
        rule.min_length = bound(deco, 0).map(|v| v as u64);
    }
    if let Some(deco) = find(&prop.decos, "MaxLength") {
        rule.max_length = bound(deco, 0).map(|v| v as u64);
    }
    if let Some(deco) = find(&prop.decos, "Min") {
        rule.minimum = bound(deco, 0);
    }
    if let Some(deco) = find(&prop.decos, "Max") {
        rule.maximum = bound(deco, 0);
    }
    if has("IsPositive") && kind == "integer" {
        rule.minimum = Some(1.0);
    }
    if let Some(pattern) = find(&prop.decos, "Matches").and_then(|d| d.first()) {
        let body = pattern.trim_start_matches('/');
        rule.pattern = Some(
            body.rsplit_once('/')
                .map(|(re, _)| re)
                .unwrap_or(body)
                .to_string(),
        );
    }
    rule
}

/// The parents of a class, itself first.
fn lineage(name: &str, world: &World) -> Vec<String> {
    let mut out = vec![name.to_string()];
    while let Some(parent) = world.parents.get(out.last().expect("one")) {
        if out.contains(parent) {
            break;
        }
        out.push(parent.clone());
    }
    out
}

/// The status Nest answers an exception with, by its own or a parent's.
fn exception_status(name: &str, world: &World) -> Option<u16> {
    let lineage = lineage(name, world);
    for each in &lineage {
        if let Some((_, status)) = NEST_EXCEPTIONS.iter().find(|(known, _)| known == each) {
            return Some(*status);
        }
    }
    if world.types.contains_key(name) || world.parents.contains_key(name) {
        return lineage
            .iter()
            .find_map(|each| contract::status_by_name(each));
    }
    None
}

/// `new HttpException('…', HttpStatus.X)`s: the status each is given.
fn http_exceptions(body: &str) -> Vec<(u16, Option<String>)> {
    let pattern = regex::Regex::new(
        r#"new\s+HttpException\s*\(\s*(?:'([^']*)'|"([^"]*)"|[^,]+)\s*,\s*(HttpStatus\.[A-Z_]+|\d{3})"#,
    )
    .expect("a valid pattern");
    pattern
        .captures_iter(body)
        .filter_map(|found| {
            let status = contract::status_of(&found[3])?;
            let message = found
                .get(1)
                .or_else(|| found.get(2))
                .map(|m| m.as_str().to_string());
            Some((status, message))
        })
        .collect()
}

/// What a controller method answers (FEAT-179).
#[allow(clippy::too_many_arguments)]
fn answers(
    world: &World,
    class: &str,
    class_decos: &[Deco],
    decos: &[Deco],
    verb: &str,
    body: &str,
    returns: &str,
    rules: &[FieldRule],
) -> Vec<Answer> {
    let mut answers = Answers::default();
    let coded = find(decos, "HttpCode")
        .and_then(|d| d.first())
        .and_then(contract::status_of);
    let success = coded.unwrap_or(if verb == "POST" { 201 } else { 200 });
    let example = if success == 204 || returns.is_empty() {
        None
    } else {
        let (inner, _) = element(returns);
        if matches!(inner.as_str(), "void" | "any" | "unknown" | "undefined") {
            None
        } else {
            let value = sample(returns, "", &[], world, 0, &mut Vec::new());
            (!matches!(value, J::Null)).then(|| value.pretty())
        }
    };
    answers.add(
        INFERRED,
        answer(
            success,
            contract::success_words(success),
            example,
            if coded.is_some() {
                "@HttpCode"
            } else {
                "the verb"
            },
        ),
    );

    if rules.iter().any(|rule| {
        rule.required
            || rule.format.is_some()
            || rule.min_length.is_some()
            || rule.max_length.is_some()
            || rule.minimum.is_some()
            || rule.maximum.is_some()
            || !rule.values.is_empty()
            || rule.pattern.is_some()
    }) {
        let messages = contract::validator_messages(rules);
        answers.add(
            INFERRED,
            answer(
                400,
                "The body breaks one of its rules",
                Some(contract::nest_error(400, None, &messages)),
                "class-validator",
            ),
        );
    }

    let guarded = find(decos, "UseGuards").or_else(|| find(class_decos, "UseGuards"));
    if guarded.is_some() {
        answers.add(
            INFERRED,
            answer(
                401,
                "Not signed in",
                Some(contract::nest_error(401, None, &[])),
                "@UseGuards",
            ),
        );
        let roles = find(decos, "Roles").or_else(|| find(class_decos, "Roles"));
        let says = match roles {
            Some(roles) if !roles.args.is_empty() => {
                format!("Signed in, but not as {}", roles.args.join(" or "))
            }
            _ => "Signed in without the right to".to_string(),
        };
        answers.add(
            INFERRED,
            answer(
                403,
                says,
                Some(contract::nest_error(403, None, &[])),
                "@UseGuards",
            ),
        );
    }

    let mut thrown = contract::thrown(body);
    let held = world.injected.get(class).cloned().unwrap_or_default();
    for (field, call) in contract::calls_on_fields(body) {
        let Some((_, ty)) = held.iter().find(|(name, _)| *name == field) else {
            continue;
        };
        let (service, _) = element(ty);
        for (_, method_body) in world
            .methods
            .get(&service)
            .into_iter()
            .flatten()
            .filter(|(name, _)| *name == call)
        {
            for found in contract::thrown(method_body) {
                if !thrown.iter().any(|(seen, _)| *seen == found.0) {
                    thrown.push(found);
                }
            }
        }
    }
    for (name, message) in thrown {
        if let Some(status) = exception_status(&name, world) {
            let says = message.clone().unwrap_or_else(|| contract::sentence(&name));
            answers.add(
                THROWN,
                answer(
                    status,
                    says,
                    Some(contract::nest_error(status, message.as_deref(), &[])),
                    format!("throws {name}"),
                ),
            );
        }
    }
    for (status, message) in http_exceptions(body) {
        let says = message
            .clone()
            .unwrap_or_else(|| contract::reason(status).to_string());
        answers.add(
            THROWN,
            answer(
                status,
                says,
                Some(contract::nest_error(status, message.as_deref(), &[])),
                "throws HttpException",
            ),
        );
    }

    for deco in decos {
        let status = if deco.name == "ApiResponse" {
            deco.object
                .iter()
                .find(|(k, _)| k == "status")
                .and_then(|(_, v)| v.text())
                .and_then(|s| contract::status_of(&s))
        } else {
            SWAGGER_RESPONSES
                .iter()
                .find(|(name, _)| *name == deco.name)
                .map(|(_, status)| *status)
        };
        if let Some(status) = status {
            let says = deco
                .object
                .iter()
                .find(|(k, _)| k == "description")
                .and_then(|(_, v)| v.text())
                .unwrap_or_else(|| contract::reason(status).to_string());
            answers.add(
                DECLARED,
                answer(status, says, None, format!("@{}", deco.name)),
            );
        }
    }

    answers.add(
        INFERRED,
        answer(
            500,
            "Something failed on the server",
            Some(contract::nest_error(500, None, &[])),
            "any endpoint",
        ),
    );
    answers.done()
}

/// The global prefix and the port, from the files that start the app.
fn bootstrap(files: &[(String, String)]) -> (String, Option<String>) {
    let prefix =
        regex::Regex::new(r#"setGlobalPrefix\(\s*['"`]([^'"`]*)['"`]"#).expect("a valid pattern");
    let listen = regex::Regex::new(r#"\.listen\(\s*(?:[\w.]+\s*(?:\?\?|\|\|)\s*)?['"]?(\d{2,5})"#)
        .expect("a valid pattern");
    let mut found_prefix = String::new();
    let mut port = None;
    for (_, text) in files {
        if !text.contains("NestFactory") {
            continue;
        }
        if let Some(m) = prefix.captures(text) {
            found_prefix = m[1].to_string();
        }
        if let Some(m) = listen.captures(text) {
            port = Some(m[1].to_string());
        }
    }
    (found_prefix, port.map(|p| format!("http://localhost:{p}")))
}

/// Every endpoint the TypeScript files' Nest controllers declare, a source
/// for each file that has any, and the `baseUrl` from `main.ts`.
pub fn read(files: &[(String, String)]) -> (Vec<Endpoint>, Vec<Source>, Option<String>) {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .is_err()
    {
        return (Vec::new(), Vec::new(), None);
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
    let (prefix, base_url) = bootstrap(files);
    let mut out = Vec::new();
    let mut sources = Vec::new();
    for file in &parsed {
        let source = &file.text;
        let start = out.len();
        for (decl, decos) in declarations(file.tree.root_node(), source) {
            if decl.kind() != "class_declaration" {
                continue;
            }
            let Some(controller) = find(&decos, "Controller") else {
                continue;
            };
            let class = decl
                .child_by_field_name("name")
                .map(|n| text(n, source).to_string())
                .unwrap_or_default();
            let base = path_arg(controller);
            // The loop below names each method's own `decos`; the class's are these.
            let class_decos = decos.clone();
            let Some(body) = decl.child_by_field_name("body") else {
                continue;
            };
            let mut pending: Vec<Deco> = Vec::new();
            for member in children(body) {
                if member.kind() == "decorator" {
                    pending.extend(decorator(member, source));
                    continue;
                }
                if member.kind() != "method_definition" {
                    pending.clear();
                    continue;
                }
                let mut decos = std::mem::take(&mut pending);
                decos.extend(own_decorators(member, source));
                let Some((verb_deco, verb)) = VERBS
                    .iter()
                    .find_map(|(name, verb)| find(&decos, name).map(|d| (d, *verb)))
                else {
                    continue;
                };
                let path = braces(&join(&[&prefix, &base, &path_arg(verb_deco)]));
                let mut declared = Declared {
                    method: verb.to_string(),
                    path: path.clone(),
                    ..Declared::default()
                };
                let mut fields: Vec<(String, J)> = Vec::new();
                let mut multipart = false;
                // The body's type, for its rules (FEAT-179).
                let mut body_ty: Option<String> = None;
                for param in member
                    .child_by_field_name("parameters")
                    .map(children)
                    .unwrap_or_default()
                {
                    let pdecos = own_decorators(param, source);
                    let pname = param
                        .child_by_field_name("pattern")
                        .map(|p| text(p, source).to_string())
                        .unwrap_or_default();
                    let ty = type_text(param.child_by_field_name("type"), source);
                    if let Some(d) = find(&pdecos, "Query") {
                        match d.first() {
                            Some(name) => declared.query.push(Param {
                                name: name.to_string(),
                                ..Param::default()
                            }),
                            // `@Query() filter: ListOrdersQuery`: its fields.
                            None => {
                                if let J::Obj(entries) =
                                    sample(&ty, "", &[], &world, 0, &mut Vec::new())
                                {
                                    for (name, value) in entries {
                                        // A string's sample is its own name, which is no
                                        // help in a query: only numbers and flags are kept.
                                        let sample = match value {
                                            J::Str(_) => String::new(),
                                            other => other.text().unwrap_or_default(),
                                        };
                                        declared.query.push(Param {
                                            name,
                                            sample,
                                            required: false,
                                        });
                                    }
                                }
                            }
                        }
                    } else if let Some(d) = find(&pdecos, "Headers") {
                        if let Some(name) = d.first() {
                            declared.headers.push(Param {
                                name: name.to_string(),
                                ..Param::default()
                            });
                        }
                    } else if let Some(d) = find(&pdecos, "Body") {
                        match d.first() {
                            Some(field) => fields.push((
                                field.to_string(),
                                sample(&ty, field, &[], &world, 0, &mut Vec::new()),
                            )),
                            None => {
                                let unwrapped = ty.trim_end_matches("[]");
                                declared.body_type = Some(
                                    unwrapped.split('<').next().unwrap_or(unwrapped).to_string(),
                                );
                                declared.body =
                                    Some(sample(&ty, "", &[], &world, 0, &mut Vec::new()));
                                body_ty = Some(ty.clone());
                            }
                        }
                    } else if let Some(d) =
                        find(&pdecos, "UploadedFile").or_else(|| find(&pdecos, "UploadedFiles"))
                    {
                        multipart = true;
                        fields.push((
                            d.first().unwrap_or(&pname).to_string(),
                            J::Str(String::new()),
                        ));
                    }
                }
                if multipart {
                    declared.consumes = Some("multipart/form-data".into());
                }
                if declared.body.is_none() && !fields.is_empty() {
                    declared.body = Some(J::Obj(fields));
                }
                let name = member
                    .child_by_field_name("name")
                    .map(|n| words_of(text(n, source)))
                    .unwrap_or_default();
                let mut hint: Vec<String> = Vec::new();
                if let Some(t) = &declared.body_type {
                    hint.push(t.clone());
                }
                let names: Vec<&str> = declared.query.iter().map(|p| p.name.as_str()).collect();
                if !names.is_empty() {
                    hint.push(names.join(", "));
                }
                declared.rules = body_ty
                    .as_deref()
                    .map(|ty| rules_of(ty, &world))
                    .unwrap_or_default();
                declared.responses = answers(
                    &world,
                    &class,
                    &class_decos,
                    &decos,
                    verb,
                    member
                        .child_by_field_name("body")
                        .map(|b| text(b, source))
                        .unwrap_or(""),
                    &type_text(member.child_by_field_name("return_type"), source),
                    &declared.rules,
                );
                let call = build(&declared, &name, Choices::default());
                out.push(Endpoint {
                    id: format!("{}#{}", file.path, out.len()),
                    key: key_of(&call),
                    source: format!("code:{}", file.path),
                    group: class.clone(),
                    file: Some(file.path.clone()),
                    line: Some(member.start_position().row as u32 + 1),
                    method: verb.to_string(),
                    path,
                    hint: hint.join(" · "),
                    call,
                    status: Status::New,
                    declared: Some(declared),
                    folder: None,
                });
            }
        }
        let count = out.len() - start;
        if count > 0 {
            sources.push(Source {
                id: format!("code:{}", file.path),
                kind: SourceKind::Nest,
                label: file
                    .path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&file.path)
                    .to_string(),
                file: Some(file.path.clone()),
                count,
                error: None,
                in_repository: true,
            });
        }
    }
    (out, sources, base_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTROLLER: &str = r#"
import { Body, Controller, Get, Headers, Param, Post, Query, UploadedFile } from '@nestjs/common';

@Controller('orders')
export class OrdersController {
  constructor(private readonly orders: OrdersService) {}

  @Get(':id')
  findOne(@Param('id') id: string, @Headers('x-tenant') tenant: string) {}

  @Get()
  findAll(@Query() query: ListQuery) {}

  @Post()
  @HttpCode(201)
  create(@Body() dto: CreateOrderDto): Promise<Order> { return null; }

  @Post(':id/receipt')
  upload(@Param('id') id: string, @UploadedFile() file: Express.Multer.File) {}

  helper() {}
}
"#;

    const DTO: &str = r#"
export enum Status { New = 'new', Paid = 'paid' }

export class LineDto {
  @IsString() sku: string;
  @IsPositive() qty: number;
}

export class CreateOrderDto {
  @ApiProperty({ example: 'AB-1' })
  @IsString()
  sku: string;

  @Min(2)
  quantity: number;

  @IsEmail()
  contact: string;

  @IsEnum(Status)
  status: Status;

  @ValidateNested({ each: true })
  @Type(() => LineDto)
  lines: LineDto[];

  @IsOptional()
  note?: string | null;

  due: Date;
  meta: Record<string, number>;
  static VERSION = 1;
}

export interface ListQuery {
  page: number;
  q?: string;
}
"#;

    const MAIN: &str = r#"
async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  app.setGlobalPrefix('api/v1');
  await app.listen(process.env.PORT ?? 3001);
}
"#;

    fn files() -> Vec<(String, String)> {
        vec![
            ("src/orders/orders.controller.ts".into(), CONTROLLER.into()),
            ("src/orders/dto.ts".into(), DTO.into()),
            ("src/main.ts".into(), MAIN.into()),
        ]
    }

    const ACCOUNTS: &str = r#"
@Controller('accounts')
@UseGuards(AuthGuard('jwt'))
export class AccountsController {
  constructor(private readonly accounts: AccountsService) {}

  @Post()
  @ApiConflictResponse({ description: 'That email has an account already' })
  open(@Body() dto: OpenAccountDto): Promise<AccountDto> {
    return this.accounts.open(dto);
  }

  @Get(':id')
  @Roles('admin')
  find(@Param('id') id: string): Promise<AccountDto> {
    return this.accounts.find(id);
  }

  @Delete(':id')
  @HttpCode(HttpStatus.NO_CONTENT)
  close(@Param('id') id: string): Promise<void> {
    if (!id) throw new HttpException('Gone for good', HttpStatus.GONE);
    return this.accounts.close(id);
  }
}

export class AccountsService {
  async open(dto: OpenAccountDto) {
    if (await this.taken(dto.email)) throw new EmailTakenException();
  }
  async find(id: string) {
    const found = await this.repo.findOne(id);
    if (!found) throw new NotFoundException(`Account ${id} not found`);
    return found;
  }
  async close(id: string) {}
}

export class EmailTakenException extends ConflictException {}

export enum Plan { Free = 'free', Pro = 'pro' }

export class AddressDto {
  @IsNotEmpty() line: string;
}

export class OpenAccountDto {
  @IsEmail() email: string;
  @Length(3, 40) name: string;
  @IsInt() @Min(18) @Max(130) age: number;
  @IsEnum(Plan) plan: Plan;
  @Matches(/^[A-Z]{2}$/) country: string;
  @IsOptional() @IsString() nickname?: string;
  @ValidateNested({ each: true }) @Type(() => AddressDto) addresses: AddressDto[];
}

export class AccountDto {
  id: string;
  email: string;
}
"#;

    /// FEAT-179: what a Nest endpoint promises, from the code around it.
    #[test]
    fn a_dto_says_its_rules_and_a_method_says_what_it_answers() {
        let (endpoints, _, _) = read(&[("src/accounts.ts".to_string(), ACCOUNTS.to_string())]);
        let declared = |method: &str, path: &str| {
            endpoints
                .iter()
                .find(|e| e.method == method && e.path == path)
                .and_then(|e| e.declared.clone())
                .expect("the endpoint")
        };

        let open = declared("POST", "/accounts");
        let rule = |path: &str| {
            open.rules
                .iter()
                .find(|r| r.path == path)
                .cloned()
                .expect(path)
        };
        assert_eq!(
            (rule("email").format.as_deref(), rule("email").required),
            (Some("email"), true)
        );
        assert_eq!(
            (rule("name").min_length, rule("name").max_length),
            (Some(3), Some(40))
        );
        let age = rule("age");
        assert_eq!(
            (age.kind.as_str(), age.minimum, age.maximum),
            ("integer", Some(18.0), Some(130.0))
        );
        assert_eq!(rule("plan").values, ["free", "pro"]);
        assert_eq!(rule("country").pattern.as_deref(), Some("^[A-Z]{2}$"));
        assert!(!rule("nickname").required);
        assert_eq!(rule("addresses").kind, "array");
        assert_eq!(rule("addresses[].line").min_length, Some(1));

        let statuses = |responses: &[Answer]| {
            responses
                .iter()
                .map(|a| (a.status, a.from.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            statuses(&open.responses),
            [
                (201, "the verb".to_string()),
                (400, "class-validator".to_string()),
                (401, "@UseGuards".to_string()),
                (403, "@UseGuards".to_string()),
                (409, "@ApiConflictResponse".to_string()),
                (500, "any endpoint".to_string())
            ]
        );
        assert!(open.responses[0]
            .body
            .as_deref()
            .unwrap_or("")
            .contains("\"email\""));
        let invalid = open.responses[1].body.as_deref().unwrap_or("");
        assert!(invalid.contains("email must be an email"), "{invalid}");
        // The author's words, and the thrown exception's body kept as the example.
        let conflict = &open.responses[4];
        assert_eq!(conflict.says, "That email has an account already");
        assert!(
            conflict
                .body
                .as_deref()
                .unwrap_or("")
                .contains("\"statusCode\": 409"),
            "{conflict:?}"
        );

        let find = declared("GET", "/accounts/{id}");
        let missing = find
            .responses
            .iter()
            .find(|a| a.status == 404)
            .expect("404");
        assert_eq!(missing.from, "throws NotFoundException");
        assert!(find
            .responses
            .iter()
            .any(|a| a.status == 403 && a.says == "Signed in, but not as admin"));

        let close = declared("DELETE", "/accounts/{id}");
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
        assert_eq!(gone.says, "Gone for good");
    }

    #[test]
    fn nest_controllers_give_their_endpoints() {
        let (endpoints, sources, base) = read(&files());
        assert_eq!(base.as_deref(), Some("http://localhost:3001"));
        let got: Vec<String> = endpoints
            .iter()
            .map(|e| format!("{} {} | {}", e.method, e.call.url, e.call.name))
            .collect();
        assert_eq!(
            got,
            [
                "GET {{baseUrl}}/api/v1/orders/{{id}} | Find one",
                "GET {{baseUrl}}/api/v1/orders?page=0&q | Find all",
                "POST {{baseUrl}}/api/v1/orders | Create",
                "POST {{baseUrl}}/api/v1/orders/{{id}}/receipt | Upload",
            ]
        );
        assert_eq!(endpoints[0].call.headers[0].name, "x-tenant");
        assert_eq!(endpoints[0].key, "GET /api/v1/orders/{}");
        assert_eq!(endpoints[2].hint, "CreateOrderDto");
        assert_eq!(
            endpoints[3].call.body.as_ref().unwrap().kind,
            crate::calls::model::BodyType::Multipart
        );
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].kind, SourceKind::Nest);
        assert_eq!(sources[0].count, 4);
    }

    #[test]
    fn a_dto_is_an_example_shaped_by_its_decorators() {
        let (endpoints, _, _) = read(&files());
        assert_eq!(
            endpoints[2].call.body.as_ref().unwrap().text.as_deref(),
            Some(
                r#"{
  "sku": "AB-1",
  "quantity": 2,
  "contact": "mira@example.com",
  "status": "new",
  "lines": [
    {
      "sku": "sku",
      "qty": 1
    }
  ],
  "note": "note",
  "due": "2026-01-31T09:30:00Z",
  "meta": {
    "key": 0
  }
}"#
            )
        );
    }
}
