use serde::de::DeserializeOwned;
use serde_json::Value;
use wslc_schema_rs::*;

const INSPECT_CONTAINER: &str = include_str!("fixtures/wslc-2.9.4/inspect-container.json");
const INSPECT_IMAGE: &str = include_str!("fixtures/wslc-2.9.4/inspect-image-hello-world.json");
const INSPECT_NETWORK: &str = include_str!("fixtures/wslc-2.9.4/inspect-network.json");
const INSPECT_VOLUME: &str = include_str!("fixtures/wslc-2.9.4/inspect-volume.json");
const SCHEMA: &str = include_str!("../generated/wslc_schema.schema.json");

fn schema_for(definition: &str) -> Value {
    let mut schema: Value = serde_json::from_str(SCHEMA).expect("generated schema is JSON");
    schema
        .as_object_mut()
        .expect("generated schema has an object root")
        .insert(
            "$ref".to_owned(),
            Value::String(format!("#/$defs/{definition}")),
        );
    schema
}

fn fixture(source: &str) -> Value {
    serde_json::from_str(source).expect("official fixture is JSON")
}

fn fragment<'a>(document: &'a Value, pointer: &str) -> &'a Value {
    document
        .pointer(pointer)
        .unwrap_or_else(|| panic!("fixture does not contain JSON pointer {pointer}"))
}

fn assert_deserializes<T: DeserializeOwned>(definition: &str, value: &Value) {
    serde_json::from_value::<T>(value.clone())
        .unwrap_or_else(|error| panic!("{definition} does not deserialize: {error}"));
}

#[test]
fn official_inspect_image_output_validates_against_json_schema() {
    let output: Value = serde_json::from_str(INSPECT_IMAGE).expect("fixture is JSON");
    let images = output.as_array().expect("wslc inspect returns an array");
    assert!(
        !images.is_empty(),
        "fixture must exercise an inspected image"
    );

    let schema = schema_for("InspectImage");
    let validator = jsonschema::validator_for(&schema).expect("generated schema compiles");

    for image in images {
        if let Err(error) = validator.validate(image) {
            panic!("official wslc image does not match InspectImage: {error}");
        }
    }
}

#[test]
fn every_schema_definition_validates_an_official_wslc_fragment() {
    let container = fixture(INSPECT_CONTAINER);
    let image = fixture(INSPECT_IMAGE);
    let network = fixture(INSPECT_NETWORK);
    let volume = fixture(INSPECT_VOLUME);

    // InspectMount and EmptyObject are structurally empty/permissive objects.
    // WSLC emitted the empty DriverOpts object; the captured container reported
    // no mounts even though it was created with --volume, so that same genuine
    // empty-object fragment exercises their wire shape without invented JSON.
    let cases = [
        ("ContainerConfig", fragment(&container, "/0/Config")),
        ("ContainerInspectState", fragment(&container, "/0/State")),
        ("EmptyObject", fragment(&volume, "/0/DriverOpts")),
        ("IPAM", fragment(&network, "/0/IPAM")),
        ("IPAMConfig", fragment(&network, "/0/IPAM/Config/0")),
        ("ImageConfig", fragment(&image, "/0/Config")),
        ("ImageRootFS", fragment(&image, "/0/RootFS")),
        ("InspectContainer", fragment(&container, "/0")),
        (
            "InspectEndpointSettings",
            fragment(
                &container,
                "/0/NetworkSettings/Networks/wslc-schema-ut-20260904",
            ),
        ),
        ("InspectHostConfig", fragment(&container, "/0/HostConfig")),
        ("InspectImage", fragment(&image, "/0")),
        ("InspectMount", fragment(&volume, "/0/DriverOpts")),
        (
            "InspectNetworkSettings",
            fragment(&container, "/0/NetworkSettings"),
        ),
        (
            "InspectPortBinding",
            fragment(&container, "/0/Ports/80~1tcp/0"),
        ),
        ("InspectVolume", fragment(&volume, "/0")),
        ("Network", fragment(&network, "/0")),
        ("Ulimit", fragment(&container, "/0/HostConfig/Ulimits/0")),
    ];

    let generated_schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let mut definitions: Vec<_> = generated_schema["$defs"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut covered: Vec<_> = cases.iter().map(|(name, _)| *name).collect();
    definitions.sort_unstable();
    covered.sort_unstable();
    assert_eq!(
        covered, definitions,
        "every $defs entry must have a fixture case"
    );

    for (definition, value) in cases {
        let schema = schema_for(definition);
        let validator = jsonschema::validator_for(&schema)
            .unwrap_or_else(|error| panic!("schema for {definition} does not compile: {error}"));
        if let Err(error) = validator.validate(value) {
            panic!("official WSLC fragment does not match {definition}: {error}");
        }
    }
}

#[test]
fn every_generated_rust_type_deserializes_an_official_wslc_fragment() {
    let container = fixture(INSPECT_CONTAINER);
    let image = fixture(INSPECT_IMAGE);
    let network = fixture(INSPECT_NETWORK);
    let volume = fixture(INSPECT_VOLUME);

    assert_deserializes::<ContainerConfig>("ContainerConfig", fragment(&container, "/0/Config"));
    assert_deserializes::<ContainerInspectState>(
        "ContainerInspectState",
        fragment(&container, "/0/State"),
    );
    assert_deserializes::<EmptyObject>("EmptyObject", fragment(&volume, "/0/DriverOpts"));
    assert_deserializes::<Ipam>("IPAM", fragment(&network, "/0/IPAM"));
    assert_deserializes::<IpamConfig>("IPAMConfig", fragment(&network, "/0/IPAM/Config/0"));
    assert_deserializes::<ImageConfig>("ImageConfig", fragment(&image, "/0/Config"));
    assert_deserializes::<ImageRootFs>("ImageRootFS", fragment(&image, "/0/RootFS"));
    assert_deserializes::<InspectContainer>("InspectContainer", fragment(&container, "/0"));
    assert_deserializes::<InspectEndpointSettings>(
        "InspectEndpointSettings",
        fragment(
            &container,
            "/0/NetworkSettings/Networks/wslc-schema-ut-20260904",
        ),
    );
    assert_deserializes::<InspectHostConfig>(
        "InspectHostConfig",
        fragment(&container, "/0/HostConfig"),
    );
    assert_deserializes::<InspectImage>("InspectImage", fragment(&image, "/0"));
    assert_deserializes::<InspectMount>("InspectMount", fragment(&volume, "/0/DriverOpts"));
    assert_deserializes::<InspectNetworkSettings>(
        "InspectNetworkSettings",
        fragment(&container, "/0/NetworkSettings"),
    );
    assert_deserializes::<InspectPortBinding>(
        "InspectPortBinding",
        fragment(&container, "/0/Ports/80~1tcp/0"),
    );
    assert_deserializes::<InspectVolume>("InspectVolume", fragment(&volume, "/0"));
    assert_deserializes::<Network>("Network", fragment(&network, "/0"));
    assert_deserializes::<Ulimit>("Ulimit", fragment(&container, "/0/HostConfig/Ulimits/0"));
}

#[test]
fn official_inspect_image_output_deserializes_to_generated_rust_types() {
    let source: Value = serde_json::from_str(INSPECT_IMAGE).expect("fixture is JSON");
    let images: Vec<InspectImage> =
        serde_json::from_str(INSPECT_IMAGE).expect("official output deserializes");

    assert_eq!(images.len(), 1);
    let image = &images[0];
    assert_eq!(image.architecture, "amd64");
    assert_eq!(image.os, "linux");
    assert_eq!(
        image.config.as_ref().unwrap().cmd.as_deref(),
        Some(["/hello".to_owned()].as_slice())
    );
    assert_eq!(image.root_fs.as_ref().unwrap().type_, "layers");
    assert_eq!(image.size, 10_072);

    let round_trip = serde_json::to_value(&images).expect("generated types serialize");
    assert_eq!(
        round_trip, source,
        "Rust types preserve the official document"
    );
}
