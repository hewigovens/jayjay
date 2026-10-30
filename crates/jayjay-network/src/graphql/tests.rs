use serde::Deserialize;
use serde_json::{Value, json};

use super::*;

#[derive(Debug, Deserialize)]
struct Data {
    value: u32,
}

fn decode(reply: Value) -> Result<Data, GraphQlResponseError> {
    serde_json::from_value::<GraphQlResponse>(reply)
        .unwrap()
        .into_data()
}

#[test]
fn only_complete_data_is_decoded() {
    assert_eq!(decode(json!({ "data": { "value": 7 } })).unwrap().value, 7);
    for reply in [json!({}), json!({ "data": null })] {
        assert!(matches!(
            decode(reply),
            Err(GraphQlResponseError::MissingData)
        ));
    }
    for data in [
        json!({}),
        json!({ "value": null }),
        json!({ "value": "seven" }),
    ] {
        assert!(matches!(
            decode(json!({ "data": data })),
            Err(GraphQlResponseError::InvalidData(_))
        ));
    }
}

#[test]
fn errors_preserve_metadata_before_decoding_partial_data() {
    for data in [Value::Null, json!({ "value": null })] {
        let reply = json!({
            "data": data,
            "errors": [
                { "message": "missing scope", "type": "INSUFFICIENT_SCOPES", "path": ["viewer"] },
                { "message": "try later", "extensions": { "code": "RATE_LIMITED" } }
            ]
        });
        let Err(GraphQlResponseError::Errors(errors)) = decode(reply) else {
            panic!("expected protocol errors rather than a data decode failure");
        };
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "missing scope");
        assert_eq!(errors[0].metadata["type"], "INSUFFICIENT_SCOPES");
        assert_eq!(errors[0].metadata["path"], json!(["viewer"]));
        assert_eq!(errors[1].metadata["extensions"]["code"], "RATE_LIMITED");
    }
}
