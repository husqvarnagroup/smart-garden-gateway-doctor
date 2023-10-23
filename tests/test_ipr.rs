use httptest::{matchers::*, responders::*, Expectation, Server};
use smart_garden_gateway_boot_analyzer::ipr::get_gw_data;

#[test]
fn test_get_gw_data() {
    let server = Server::run();

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/ipr/lm.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let file_content = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));
    let resp_lm = status_code(200).body(file_content);

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/ipr/gw.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let file_content = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));
    let resp_gw = status_code(200).body(file_content);

    server.expect(
        Expectation::matching(request::method_path(
            "GET",
            "/v2/products/b7ece5ec-10bf-4b75-9f57-62cd62dfd284",
        ))
        .respond_with(resp_lm),
    );

    server.expect(
        Expectation::matching(request::method_path(
            "GET",
            "/v2/products/1b54e2fb-a0d1-4200-ac3c-ab604d834875",
        ))
        .respond_with(resp_gw),
    );

    match get_gw_data(
        server.url_str("/").strip_suffix('/').unwrap(),
        "key",
        "b7ece5ec-10bf-4b75-9f57-62cd62dfd284",
    ) {
        Ok(gw_data) => assert!(gw_data.status == "MANUFACTURED"),
        Err(e) => panic!("{e}"),
    }
}
