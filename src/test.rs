#[cfg(test)]
use warp::http::StatusCode;
use warp::test::request;
use warp::Filter;

#[tokio::test]
async fn try_root() {

    let root = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file("static/login.html"));

    let response = request()
        .method("GET")
        .path("/")
        .reply(&root)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}
