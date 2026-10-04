//! Prints the schemas `ToSchema` generated for `WatchStatus` and `Anime`.

use p3_03_03_api_contracts_and_openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    for name in ["WatchStatus", "Anime"] {
        println!("--- {name}");
        let schema = &doc["components"]["schemas"][name];
        println!("{}", serde_json::to_string_pretty(schema).unwrap());
    }
}
