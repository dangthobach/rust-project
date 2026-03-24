use axum::{
    extract::Request,
    http::header::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Adds controlled deprecation headers for legacy `/api/files/*` routes.
pub async fn legacy_files_api_deprecation(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        HeaderName::from_static("deprecation"),
        HeaderValue::from_static("true"),
    );
    res.headers_mut().insert(
        HeaderName::from_static("sunset"),
        HeaderValue::from_static("Wed, 31 Dec 2026 23:59:59 GMT"),
    );
    res.headers_mut().insert(
        HeaderName::from_static("link"),
        HeaderValue::from_static("</api/fs/files>; rel=\"successor-version\""),
    );
    res
}
