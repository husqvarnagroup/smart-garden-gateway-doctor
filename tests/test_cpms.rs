use httpmock::prelude::*;
use smart_garden_gateway_boot_analyzer::cpms::get_gw_data;

#[test]
fn test_get_gw_data() {
    let server = MockServer::start();

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/cpms/lm.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let resp_lm = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));

    let file_path = std::path::PathBuf::from(format!(
        "{}/tests/data/cpms/gw.json",
        env!("CARGO_MANIFEST_DIR")
    ));
    let resp_gw = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", &file_path.display()));

    let lm_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/iprconfig/37e12251-11cc-487a-a74f-4d50a41f3815");
        then.status(200)
            .header("content-type", "application/json")
            .body(resp_lm);
    });

    let gw_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/iprconfig/12345678-9abc-def0-1234-56789abcdef0");
        then.status(200)
            .header("content-type", "application/json")
            .body(resp_gw);
    });

    match get_gw_data(
        server.url("/").strip_suffix('/').unwrap(),
        "user",
        "password",
        "37e12251-11cc-487a-a74f-4d50a41f3815",
    ) {
        Ok(gw_data) => {
            assert_eq!(gw_data.status, "NEW");
        }
        Err(e) => panic!("{e}"),
    }

    lm_mock.assert();
    gw_mock.assert();
}
