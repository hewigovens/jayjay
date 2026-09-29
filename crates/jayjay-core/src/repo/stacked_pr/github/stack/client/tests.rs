use super::ApiError;

#[test]
fn parses_gh_http_status_and_api_message() {
    let error = ApiError::from_text(
        r#"{"message":"Pull requests must form a stack","status":"422"}"#,
        "",
    );

    assert_eq!(error.status(), Some(422));
    assert_eq!(error.to_string(), "Pull requests must form a stack");
}
