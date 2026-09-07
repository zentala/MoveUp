//! Tests for [`crate::remote_auth`] (E021-T03).

use axum::http::{HeaderMap, HeaderValue, StatusCode};

use crate::remote_auth::{constant_time_eq, require_token, TOKEN_HEADER};

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(TOKEN_HEADER, HeaderValue::from_str(token).expect("ascii"));
    headers
}

#[test]
fn e021_t03_matching_token_is_accepted() {
    assert_eq!(
        require_token(&headers_with("s3cret"), Some("s3cret")),
        Ok(())
    );
}

#[test]
fn e021_t03_wrong_token_is_unauthorized() {
    assert_eq!(
        require_token(&headers_with("nope"), Some("s3cret")),
        Err(StatusCode::UNAUTHORIZED)
    );
}

#[test]
fn e021_t03_missing_header_is_unauthorized() {
    assert_eq!(
        require_token(&HeaderMap::new(), Some("s3cret")),
        Err(StatusCode::UNAUTHORIZED)
    );
}

#[test]
fn e021_t03_unconfigured_token_closes_the_inlet_rather_than_opening_it() {
    // The failure this guards: an unset secret making every request succeed.
    assert_eq!(
        require_token(&headers_with("anything"), None),
        Err(StatusCode::SERVICE_UNAVAILABLE)
    );
    assert_eq!(
        require_token(&headers_with(""), Some("")),
        Err(StatusCode::SERVICE_UNAVAILABLE)
    );
}

#[test]
fn e021_t03_non_ascii_header_value_is_rejected_not_panicking() {
    let mut headers = HeaderMap::new();
    headers.insert(
        TOKEN_HEADER,
        HeaderValue::from_bytes(&[0xff, 0xfe]).expect("opaque bytes are a valid header value"),
    );
    assert_eq!(
        require_token(&headers, Some("s3cret")),
        Err(StatusCode::UNAUTHORIZED)
    );
}

#[test]
fn e021_t03_constant_time_eq_agrees_with_plain_equality() {
    let cases: &[(&[u8], &[u8])] = &[
        (b"", b""),
        (b"a", b"a"),
        (b"a", b"b"),
        (b"", b"\0"),
        (b"\0", b""),
        (b"abc", b"abcd"),
        (b"abcd", b"abc"),
        (b"token-with-\xc3\xa9", b"token-with-\xc3\xa9"),
    ];
    for (a, b) in cases {
        assert_eq!(
            constant_time_eq(a, b),
            a == b,
            "constant_time_eq({a:?}, {b:?}) disagreed with =="
        );
    }
}

#[test]
fn e021_t03_constant_time_eq_is_not_fooled_by_length_multiples_of_256() {
    // A naive `(a.len() ^ b.len()) as u8` accumulator collapses to 0 here.
    let short = vec![b'x'; 1];
    let long = vec![b'x'; 257];
    assert!(!constant_time_eq(&short, &long));
}
