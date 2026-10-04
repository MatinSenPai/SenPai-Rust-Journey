//! Prints three slices of the OpenAPI document that `ApiDoc` generates.

use p3_03_03_api_contracts_and_openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    // The same JSON a client downloads, as a plain `serde_json::Value`.
    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    println!("openapi version: {}", doc["openapi"]);
    let paths: Vec<&String> = doc["paths"].as_object().unwrap().keys().collect();
    println!("paths: {paths:?}");
    let schemas: Vec<&String> = doc["components"]["schemas"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    println!("schemas: {schemas:?}");

    println!("--- GET /anime/{{id}} responses");
    let responses = &doc["paths"]["/anime/{id}"]["get"]["responses"];
    println!("{}", serde_json::to_string_pretty(responses).unwrap());
}
