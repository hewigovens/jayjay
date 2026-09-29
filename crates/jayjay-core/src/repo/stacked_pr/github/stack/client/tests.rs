use super::{ApiError, pull_request_fields};

#[test]
fn builds_typed_gh_array_fields() {
    assert_eq!(
        pull_request_fields(&[10, 20]).collect::<Vec<_>>(),
        ["pull_requests[]=10", "pull_requests[]=20"]
    );
}

#[test]
fn parses_gh_http_status_and_api_message() {
    let error = ApiError::from_text(
        r#"{"message":"Pull requests must form a stack","status":"422"}"#,
        "",
    );

    assert_eq!(error.status(), Some(422));
    assert_eq!(error.to_string(), "Pull requests must form a stack");
}
