use anyhow::{Result, bail};
use serde_json::{Map, Value, json};

use crate::model::{CppType, Declaration};

pub fn generate(declarations: &[Declaration], version: &str) -> Result<Value> {
    let names: std::collections::BTreeSet<_> =
        declarations.iter().map(|d| d.name.as_str()).collect();
    let mut defs = Map::new();
    defs.insert(
        "EmptyObject".to_owned(),
        json!({ "type": "object", "additionalProperties": false, "default": {} }),
    );
    for declaration in declarations {
        let mut properties = Map::new();
        for field in &declaration.fields {
            let mut property = type_schema(&field.ty, &names)?;
            if let Some(description) = &field.description {
                property
                    .as_object_mut()
                    .expect("type schemas are objects")
                    .insert("description".to_owned(), Value::String(description.clone()));
            }
            properties.insert(field.name.clone(), property);
        }
        defs.insert(
            declaration.name.clone(),
            json!({
                "type": "object",
                "properties": properties,
                "additionalProperties": true,
                "default": {}
            }),
        );
    }
    Ok(json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://github.com/mveril/wslc_schema_rs/schema/{version}/wslc_schema.schema.json"),
        "title": format!("WSL container schema {version}"),
        "description": "Generated from microsoft/WSL src/windows/inc/wslc_schema.h",
        "$defs": defs
    }))
}

fn type_schema(ty: &CppType, names: &std::collections::BTreeSet<&str>) -> Result<Value> {
    Ok(match ty {
        CppType::String => json!({ "type": "string", "default": "" }),
        CppType::Bool => json!({ "type": "boolean", "default": false }),
        CppType::I32 => json!({
            "type": "integer", "minimum": i32::MIN, "maximum": i32::MAX, "default": 0
        }),
        CppType::I64 => json!({
            "type": "integer", "minimum": i64::MIN, "maximum": i64::MAX, "default": 0
        }),
        CppType::Vector(inner) => {
            json!({ "type": "array", "items": type_schema(inner, names)?, "default": [] })
        }
        CppType::Map(inner) => json!({
            "type": "object", "additionalProperties": type_schema(inner, names)?, "default": {}
        }),
        CppType::Optional(inner) => json!({
            "anyOf": [type_schema(inner, names)?, { "type": "null" }], "default": null
        }),
        CppType::Named(name) => {
            if name != "EmptyObject" && !names.contains(name.as_str()) {
                bail!("reference to unknown C++ type {name}");
            }
            json!({ "$ref": format!("#/$defs/{name}"), "default": {} })
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn committed_schema_is_well_formed_and_complete() {
        let schema = include_str!("../../wslc_schema_rs/generated/wslc_schema.schema.json");
        let root: schemars::schema::RootSchema = serde_json::from_str(schema).unwrap();
        let expected = [
            "ContainerConfig",
            "ContainerInspectState",
            "EmptyObject",
            "IPAM",
            "IPAMConfig",
            "ImageConfig",
            "ImageRootFS",
            "InspectContainer",
            "InspectEndpointSettings",
            "InspectHostConfig",
            "InspectImage",
            "InspectMount",
            "InspectNetworkSettings",
            "InspectPortBinding",
            "InspectVolume",
            "Network",
            "Ulimit",
        ];
        assert_eq!(root.definitions.len(), expected.len());
        for name in expected {
            assert!(
                root.definitions.contains_key(name),
                "missing definition {name}"
            );
        }
    }
}
