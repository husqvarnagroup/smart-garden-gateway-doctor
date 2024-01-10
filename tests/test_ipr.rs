use httpmock::prelude::*;
use smart_garden_gateway_boot_analyzer::ipr::get_gw_data;

#[test]
fn test_get_gw_data() {
    let server = MockServer::start();

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/ipr/lm.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let resp_lm = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/ipr/gw.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let resp_gw = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));

    let lm_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/products/b7ece5ec-10bf-4b75-9f57-62cd62dfd284");
        then.status(200)
            .header("content-type", "application/json")
            .body(resp_lm);
    });

    let gw_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v2/products/12345678-9abc-def0-1234-56789abcdef0");
        then.status(200)
            .header("content-type", "application/json")
            .body(resp_gw);
    });

    match get_gw_data(
        server.url("/").strip_suffix('/').unwrap(),
        "key",
        "b7ece5ec-10bf-4b75-9f57-62cd62dfd284",
    ) {
        Ok(gw_data) => {
            assert_eq!(gw_data.status, "MANUFACTURED");
        }
        Err(e) => panic!("{e}"),
    }

    lm_mock.assert();
    gw_mock.assert();
}
