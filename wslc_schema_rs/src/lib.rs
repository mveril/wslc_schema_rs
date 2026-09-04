//! Serde types for the JSON documents described by WSL's `wslc_schema.h`.
//!
//! The types in this crate are generated from a versioned JSON Schema. Their
//! Rust field names are idiomatic `snake_case`; Serde preserves WSL's original
//! JSON property names.

#![forbid(unsafe_code)]

#[allow(clippy::derivable_impls)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/wslc_schema.rs"
    ));
}

pub use generated::*;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn missing_fields_use_wsl_defaults() {
        let state: ContainerInspectState = serde_json::from_value(json!({})).unwrap();
        assert_eq!(state, ContainerInspectState::default());
        assert!(!state.running);
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn optionals_accept_missing_and_null() {
        let missing: ImageConfig = serde_json::from_value(json!({})).unwrap();
        let null: ImageConfig = serde_json::from_value(json!({ "Cmd": null })).unwrap();
        assert_eq!(missing.cmd, None);
        assert_eq!(null.cmd, None);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let binding: InspectPortBinding = serde_json::from_value(json!({
            "HostIp": "127.0.0.1",
            "HostPort": "8080",
            "FutureField": true
        }))
        .unwrap();
        assert_eq!(binding.host_ip, "127.0.0.1");
        assert_eq!(binding.host_port, "8080");
    }

    #[test]
    fn serialization_uses_original_wsl_names() {
        let binding = InspectPortBinding {
            host_ip: "127.0.0.1".to_owned(),
            host_port: "8080".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(binding).unwrap(),
            json!({ "HostIp": "127.0.0.1", "HostPort": "8080" })
        );
    }

    #[test]
    fn serialization_matches_nlohmann_default_shape() {
        assert_eq!(
            serde_json::to_value(ContainerConfig::default()).unwrap(),
            json!({
                "Cmd": null,
                "Entrypoint": null,
                "Env": null,
                "User": "",
                "WorkingDir": ""
            })
        );

        let value = serde_json::to_value(InspectContainer::default()).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 11);
        assert_eq!(object["Mounts"], json!([]));
        assert_eq!(object["Ports"], json!({}));
        assert_eq!(object["Labels"], json!({}));
    }

    #[test]
    fn json_round_trip_preserves_wsl_values() {
        let input = json!({
            "Id": "container-id",
            "Name": "example",
            "Config": { "Cmd": ["echo", "hello"], "Env": null },
            "Ports": { "80/tcp": [{ "HostIp": "127.0.0.1", "HostPort": "8080" }] }
        });
        let decoded: InspectContainer = serde_json::from_value(input).unwrap();
        let output = serde_json::to_value(decoded).unwrap();
        assert_eq!(output["Id"], "container-id");
        assert_eq!(output["Config"]["Cmd"], json!(["echo", "hello"]));
        assert_eq!(output["Config"]["Env"], json!(null));
        assert_eq!(output["Ports"]["80/tcp"][0]["HostPort"], "8080");
    }

    #[test]
    fn empty_object_matches_cpp_permissive_decoder_and_canonical_encoder() {
        for input in [
            json!({}),
            json!({ "ignored": true }),
            json!(42),
            json!(null),
        ] {
            let value: EmptyObject = serde_json::from_value(input).unwrap();
            assert_eq!(serde_json::to_value(value).unwrap(), json!({}));
        }
    }
}
