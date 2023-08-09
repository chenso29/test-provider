use std::collections::BTreeMap;

use fiberplane_pdk::prelude::*;
use serde::Deserialize;
use serde_json;

pub const RETOOL_EMBED_APP: &str = "x-showcase-cells";

pub const SHOWCASE_MIME_TYPE: &str = "application/vnd.fiberplane.providers.sample.showcase";

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Debug, Deserialize)]
struct ApiResponse {
    embed_url: String,
}

async fn get_link() -> Result<String> {
    let body_data: serde_json::Value = serde_json::json!({
        "landingPageUuid": "b856ef26-304c-11ee-86b5-5f6067919839",
        "groupIds": [1],
        "externalIdentifier": "Test Provider"
    });

    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        "Bearer retool_01h6qxqrtae3r8xvnz7zy0x9z6".to_string(),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request = HttpRequest::post(
        "https://retool.slexn.com/api/embed-url/external-user",
        body_data.to_string(),
    )
    .with_headers(headers);

    let response = make_http_request(request).await?;

    let response_data: ApiResponse =
        serde_json::from_slice(&response.body).map_err(|e| Error::Deserialization {
            message: format!("Could not deserialize payload: {e:?}"),
        })?;

    Ok(response_data.embed_url)
}

async fn get_embed_app() -> Result<String> {
    let embed_url = get_link().await?;

    let request = HttpRequest::get(&embed_url);
    let response = make_http_request(request).await?;

    let body_as_string = String::from_utf8_lossy(&response.body).to_string();

    Ok(body_as_string)
}

async fn js_to_blob() -> Result<Blob> {
    let response = get_embed_app().await?;
    let cells = vec![Cell::Text(
        TextCell::builder()
            .content(response)
            .formatting(Formatting::default())
            .build(),
    )];

    Cells(cells).to_blob()
}

fn check_status() -> Result<Blob> {
    ProviderStatus::builder()
        .status(Ok(()))
        .version(COMMIT_HASH.to_owned())
        .built_at(BUILD_TIMESTAMP.to_owned())
        .build()
        .to_blob()
}

pdk_query_types! {
    RETOOL_EMBED_APP => {
        label: "Retool: show chart",
        handler: js_to_blob().await,
        supported_mime_types: [CELLS_MIME_TYPE]
    },
    STATUS_QUERY_TYPE => {
        handler: check_status(),
        supported_mime_types: [STATUS_MIME_TYPE]
    }
}
