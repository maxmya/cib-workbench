// SPDX-License-Identifier: MIT

//! Which service an endpoint belongs to (FEAT-177).
//!
//! A repository often holds more than one service — a Maven module per
//! microservice, a NestJS monorepo's `apps/` — and Calls drew every
//! controller side by side with nothing to say whose it was. A service is
//! the folder a build file stands in: the nearest `pom.xml`,
//! `build.gradle`, `build.gradle.kts` or `package.json` above a
//! controller, named as the build names it (Maven's own `artifactId`,
//! `package.json`'s `name`) and said as what it is built as — Spring Boot,
//! NestJS — where the build says so. In a NestJS monorepo, whose apps share
//! the root's `package.json`, each `apps/<name>` is a service of its own.

use std::collections::HashSet;

use serde::Serialize;

/// A service in the repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    /// What it is built as: `Spring Boot`, `NestJS`, `Express`, `Node`,
    /// `Maven`, `Gradle`.
    pub kind: String,
    /// Its folder from the repository's root, with `/`; empty for the root.
    pub root: String,
}

const BUILD_FILES: [&str; 4] = [
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "package.json",
];

/// The folders above `file`, nearest first, ending with the root as `""`.
fn folders(file: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut at = file;
    while let Some((parent, _)) = at.rsplit_once('/') {
        out.push(parent);
        at = parent;
    }
    out.push("");
    out
}

fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}/{name}")
    }
}

/// The text between the first `<tag>` and its `</tag>` in `text`.
fn element<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(text[start..end].trim())
}

/// A Maven module's own `artifactId`: the first one outside its `<parent>`
/// and its `<dependencies>`.
fn artifact_id(pom: &str) -> Option<String> {
    let mut text = pom.to_string();
    for tag in [
        "parent",
        "dependencies",
        "dependencyManagement",
        "build",
        "profiles",
    ] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        while let (Some(start), Some(end)) = (text.find(&open), text.find(&close)) {
            if end < start {
                break;
            }
            text.replace_range(start..end + close.len(), "");
        }
    }
    element(&text, "artifactId")
        .map(str::to_string)
        .filter(|id| !id.is_empty())
}

/// `rootProject.name = 'orders'` in a Gradle settings file.
fn gradle_name(settings: &str) -> Option<String> {
    settings.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("rootProject.name")?;
        let value = rest.trim().strip_prefix('=')?.trim();
        let value = value.trim_matches(['"', '\'']).trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn last(folder: &str, fallback: &str) -> String {
    folder
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

/// The service a build file in `folder` makes, read with `read`.
fn service_in(
    folder: &str,
    build: &str,
    read: &dyn Fn(&str) -> Option<String>,
    repository: &str,
) -> Service {
    let text = read(&join(folder, build)).unwrap_or_default();
    let (name, kind) = match build {
        "pom.xml" => (
            artifact_id(&text).unwrap_or_else(|| last(folder, repository)),
            if text.contains("spring-boot") {
                "Spring Boot"
            } else {
                "Maven"
            },
        ),
        "package.json" => {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let name = json["name"]
                .as_str()
                .map(str::to_string)
                .filter(|name| !name.is_empty());
            let kind = if text.contains("\"@nestjs/core\"") {
                "NestJS"
            } else if text.contains("\"express\"") {
                "Express"
            } else {
                "Node"
            };
            (name.unwrap_or_else(|| last(folder, repository)), kind)
        }
        _ => {
            let name = if folder.is_empty() {
                read("settings.gradle")
                    .or_else(|| read("settings.gradle.kts"))
                    .and_then(|settings| gradle_name(&settings))
            } else {
                None
            };
            (
                name.unwrap_or_else(|| last(folder, repository)),
                if text.contains("org.springframework.boot") {
                    "Spring Boot"
                } else {
                    "Gradle"
                },
            )
        }
    };
    Service {
        name,
        kind: kind.to_string(),
        root: folder.to_string(),
    }
}

/// The service `file` is in: the nearest build file above it, or a NestJS
/// monorepo's app. `listed` is every file in the working tree.
pub fn find(
    file: &str,
    listed: &HashSet<&str>,
    read: &dyn Fn(&str) -> Option<String>,
    repository: &str,
) -> Option<Service> {
    for folder in folders(file) {
        let Some(build) = BUILD_FILES
            .iter()
            .find(|build| listed.contains(join(folder, build).as_str()))
        else {
            continue;
        };
        // A NestJS monorepo: one package.json, an app in each apps/<name>.
        if *build == "package.json" && listed.contains(join(folder, "nest-cli.json").as_str()) {
            let inside = if folder.is_empty() {
                file
            } else {
                &file[folder.len() + 1..]
            };
            if let Some(app) = inside
                .strip_prefix("apps/")
                .and_then(|rest| rest.split('/').next())
            {
                if inside.len() > "apps/".len() + app.len() {
                    return Some(Service {
                        name: app.to_string(),
                        kind: "NestJS".into(),
                        root: join(folder, &format!("apps/{app}")),
                    });
                }
            }
        }
        return Some(service_in(folder, build, read, repository));
    }
    None
}

/// The service `file` is in, of `services`: the one with the longest root
/// above it.
pub fn of<'a>(services: &'a [Service], file: &str) -> Option<&'a Service> {
    services
        .iter()
        .filter(|service| {
            service.root.is_empty() || file.starts_with(&format!("{}/", service.root))
        })
        .max_by_key(|service| service.root.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn tree(
        files: &[(&'static str, &'static str)],
    ) -> (HashSet<&'static str>, HashMap<&'static str, &'static str>) {
        (
            files.iter().map(|(path, _)| *path).collect(),
            files.iter().copied().collect(),
        )
    }

    #[test]
    fn a_maven_module_is_its_own_artifact_and_says_it_is_spring_boot() {
        let (listed, texts) = tree(&[
            ("pom.xml", "<project><artifactId>platform</artifactId><packaging>pom</packaging></project>"),
            (
                "services/orders/pom.xml",
                "<project><parent><artifactId>platform</artifactId></parent><artifactId>orders-service</artifactId><dependencies><dependency><artifactId>spring-boot-starter-web</artifactId></dependency></dependencies></project>",
            ),
            ("services/orders/src/main/java/OrderController.java", ""),
        ]);
        let read = |path: &str| texts.get(path).map(|text| text.to_string());
        let found = find(
            "services/orders/src/main/java/OrderController.java",
            &listed,
            &read,
            "repo",
        )
        .expect("a service");
        assert_eq!(
            found,
            Service {
                name: "orders-service".into(),
                kind: "Spring Boot".into(),
                root: "services/orders".into()
            }
        );
    }

    #[test]
    fn a_nest_app_is_named_by_its_package_and_each_monorepo_app_is_its_own() {
        let (listed, texts) = tree(&[
            (
                "billing/package.json",
                r#"{ "name": "billing-api", "dependencies": { "@nestjs/core": "10" } }"#,
            ),
            ("billing/src/invoice.controller.ts", ""),
            (
                "package.json",
                r#"{ "name": "platform", "dependencies": { "@nestjs/core": "10" } }"#,
            ),
            ("nest-cli.json", r#"{ "monorepo": true }"#),
            ("apps/orders/src/orders.controller.ts", ""),
        ]);
        let read = |path: &str| texts.get(path).map(|text| text.to_string());
        let billing =
            find("billing/src/invoice.controller.ts", &listed, &read, "repo").expect("billing");
        assert_eq!(
            (
                billing.name.as_str(),
                billing.kind.as_str(),
                billing.root.as_str()
            ),
            ("billing-api", "NestJS", "billing")
        );
        let orders = find(
            "apps/orders/src/orders.controller.ts",
            &listed,
            &read,
            "repo",
        )
        .expect("orders");
        assert_eq!(
            (
                orders.name.as_str(),
                orders.kind.as_str(),
                orders.root.as_str()
            ),
            ("orders", "NestJS", "apps/orders")
        );
    }

    #[test]
    fn gradle_at_the_root_is_named_by_its_settings() {
        let (listed, texts) = tree(&[
            (
                "build.gradle.kts",
                "plugins { id(\"org.springframework.boot\") }",
            ),
            ("settings.gradle.kts", "rootProject.name = \"payments\""),
            ("src/main/java/PayController.java", ""),
        ]);
        let read = |path: &str| texts.get(path).map(|text| text.to_string());
        let found =
            find("src/main/java/PayController.java", &listed, &read, "repo").expect("a service");
        assert_eq!(
            (
                found.name.as_str(),
                found.kind.as_str(),
                found.root.as_str()
            ),
            ("payments", "Spring Boot", "")
        );
    }

    #[test]
    fn a_file_is_in_the_service_whose_root_is_nearest() {
        let services = vec![
            Service {
                name: "platform".into(),
                kind: "Maven".into(),
                root: String::new(),
            },
            Service {
                name: "orders".into(),
                kind: "Spring Boot".into(),
                root: "services/orders".into(),
            },
        ];
        assert_eq!(
            of(&services, "services/orders/src/A.java").map(|s| s.name.as_str()),
            Some("orders")
        );
        assert_eq!(
            of(&services, "services/ordersx/src/A.java").map(|s| s.name.as_str()),
            Some("platform")
        );
    }
}
