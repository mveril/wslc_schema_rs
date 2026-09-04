use anyhow::{Context, Result, bail};
use tree_sitter::{Node, Parser};

use crate::model::{CppType, Declaration, Field};

pub fn parse(source: &str) -> Result<Vec<Declaration>> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_cpp::LANGUAGE.into())?;
    let tree = parser
        .parse(source, None)
        .context("tree-sitter failed to parse header")?;
    if tree.root_node().has_error() {
        bail!("wslc_schema.h contains C++ syntax errors");
    }
    let mut declarations = Vec::new();
    visit(
        tree.root_node(),
        source.as_bytes(),
        false,
        &mut declarations,
    )?;
    if declarations.is_empty() {
        bail!("no struct declarations found in wslc_schema.h");
    }
    Ok(declarations)
}

fn visit(
    node: Node<'_>,
    source: &[u8],
    inside_schema_namespace: bool,
    out: &mut Vec<Declaration>,
) -> Result<()> {
    let inside_schema_namespace = if node.kind() == "namespace_definition" {
        let declaration = text(node, source)?;
        if !declaration
            .trim_start()
            .starts_with("namespace wsl::windows::common::wslc_schema")
        {
            return Ok(());
        }
        true
    } else {
        inside_schema_namespace
    };
    if inside_schema_namespace && node.kind() == "struct_specifier" {
        out.push(parse_struct(node, source)?);
        return Ok(());
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit(child, source, inside_schema_namespace, out)?;
    }
    Ok(())
}

fn parse_struct(node: Node<'_>, source: &[u8]) -> Result<Declaration> {
    let name_node = node
        .child_by_field_name("name")
        .context("struct without a name")?;
    let name = text(name_node, source)?.to_owned();
    let body = node
        .child_by_field_name("body")
        .context("struct without a body")?;
    let mut fields = Vec::new();
    let mut comments = Vec::new();
    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        if child.kind() == "comment" {
            comments.push(comment_text(text(child, source)?));
            continue;
        }
        if child.kind() != "field_declaration" {
            comments.clear();
            continue;
        }
        let raw = text(child, source)?.trim().trim_end_matches(';').trim();
        let split = raw
            .rfind(|character: char| character.is_whitespace())
            .with_context(|| format!("unsupported field declaration in {name}: {raw}"))?;
        let ty = raw[..split].trim();
        let field_name = raw[split..].trim().trim_end_matches("{}");
        if field_name.is_empty()
            || !field_name
                .chars()
                .all(|c| c == '_' || c.is_ascii_alphanumeric())
        {
            bail!("unsupported field declarator in {name}: {raw}");
        }
        fields.push(Field {
            name: field_name.to_owned(),
            ty: parse_type(ty)?,
            description: (!comments.is_empty()).then(|| comments.join("\n")),
        });
        comments.clear();
    }
    validate_macro(&name, &fields, text(node, source)?)?;
    Ok(Declaration { name, fields })
}

fn comment_text(raw: &str) -> String {
    raw.lines()
        .map(|line| {
            line.trim()
                .trim_start_matches("//")
                .trim_start_matches("/*")
                .trim_end_matches("*/")
                .trim_start_matches('*')
                .trim()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_type(raw: &str) -> Result<CppType> {
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    match compact.as_str() {
        "std::string" => Ok(CppType::String),
        "bool" => Ok(CppType::Bool),
        "int" => Ok(CppType::I32),
        "int64_t" | "std::int64_t" => Ok(CppType::I64),
        _ => {
            for (prefix, constructor) in [
                (
                    "std::vector<",
                    CppType::Vector as fn(Box<CppType>) -> CppType,
                ),
                ("std::optional<", CppType::Optional),
                ("std::map<std::string,", CppType::Map),
            ] {
                if compact.starts_with(prefix) && compact.ends_with('>') {
                    let inner = &compact[prefix.len()..compact.len() - 1];
                    return Ok(constructor(Box::new(parse_type(inner)?)));
                }
            }
            if compact
                .chars()
                .all(|c| c == '_' || c.is_ascii_alphanumeric())
            {
                Ok(CppType::Named(compact))
            } else {
                bail!("unsupported C++ type: {raw}")
            }
        }
    }
}

fn validate_macro(name: &str, fields: &[Field], body: &str) -> Result<()> {
    const MACRO: &str = "NLOHMANN_DEFINE_TYPE_INTRUSIVE_WITH_DEFAULT";
    let start = body
        .find(MACRO)
        .with_context(|| format!("{name} has no {MACRO} macro"))?;
    let arguments = &body[start + MACRO.len()..];
    let open = arguments
        .find('(')
        .context("serialization macro has no argument list")?;
    let close = arguments[open + 1..]
        .find(')')
        .map(|offset| open + 1 + offset)
        .context("serialization macro has no closing parenthesis")?;
    let names: Vec<_> = arguments[open + 1..close]
        .split(',')
        .map(str::trim)
        .collect();
    let expected: Vec<_> = std::iter::once(name)
        .chain(fields.iter().map(|field| field.name.as_str()))
        .collect();
    if names != expected {
        bail!(
            "{name} fields do not match its serialization macro: expected {expected:?}, found {names:?}"
        );
    }
    Ok(())
}

fn text<'a>(node: Node<'_>, source: &'a [u8]) -> Result<&'a str> {
    Ok(std::str::from_utf8(&source[node.byte_range()])?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_nested_types() {
        let ty = parse_type("std::optional<std::map<std::string, std::vector<Foo>>>").unwrap();
        assert_eq!(
            ty,
            CppType::Optional(Box::new(CppType::Map(Box::new(CppType::Vector(Box::new(
                CppType::Named("Foo".into())
            ))))))
        );
    }

    #[test]
    fn rejects_unknown_pointer_types() {
        assert!(parse_type("char*").is_err());
    }

    #[test]
    fn only_reads_the_wslc_schema_namespace() {
        let source = r#"
            namespace ignored { struct Nope { int Value{}; }; }
            namespace wsl::windows::common::wslc_schema {
                struct Good {
                    int Value{};
                    NLOHMANN_DEFINE_TYPE_INTRUSIVE_WITH_DEFAULT(Good, Value);
                };
            }
        "#;
        let declarations = parse(source).unwrap();
        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].name, "Good");
    }

    #[test]
    fn preserves_field_comments() {
        let source = r#"
            namespace wsl::windows::common::wslc_schema {
                struct Mount {
                    // Whether the mount is writable.
                    bool ReadWrite{};
                    NLOHMANN_DEFINE_TYPE_INTRUSIVE_WITH_DEFAULT(Mount, ReadWrite);
                };
            }
        "#;
        let declarations = parse(source).unwrap();
        assert_eq!(
            declarations[0].fields[0].description.as_deref(),
            Some("Whether the mount is writable.")
        );
    }
}
