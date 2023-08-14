mod clv;

use chrono::{NaiveDate, NaiveDateTime};
use clv::{PrometheusPoint, RangeVector};
use fiberplane_pdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const RETOOL_EMBED_APP: &str = "x-showcase-cells";

pub const SHOWCASE_MIME_TYPE: &str = "application/vnd.fiberplane.providers.sample.showcase";

static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
static BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");

#[derive(Debug, Serialize, Deserialize)]
struct OriginalObject {
    x: Vec<String>,
    y: Vec<i32>,
    name: String,
    #[serde(rename = "type")]
    object_type: String,
}

async fn fetch_graph_data() -> Result<Blob> {
    let request = HttpRequest::get(
        "http://115.178.76.46:31998/charts/totalchart?intervalE=years%20ago&intervalS=1",
    );

    let response = make_http_request(request).await?;
    let body_as_string = String::from_utf8_lossy(&response.body).to_string();
    let original_objects: Vec<OriginalObject> = serde_json::from_str(&body_as_string)?;

    let mut transformed_objects: Vec<RangeVector> = Vec::new();

    for obj in original_objects {
        let mut metric = BTreeMap::new();
        metric.insert("__name__".to_string(), obj.name.clone());
        metric.insert("job".to_string(), "clv".to_string());
        metric.insert("instance".to_string(), obj.name.clone());

        let values = obj
            .x
            .iter()
            .zip(&obj.y)
            .filter_map(|(time, value)| {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(time, "%Y-%m-%d") {
                    if let Some(start_of_day) = chrono::NaiveTime::from_hms_opt(0, 0, 0) {
                        let time_unix =
                            (NaiveDateTime::new(parsed_date, start_of_day)).timestamp() as f64;
                        Some(PrometheusPoint(time_unix, value.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        transformed_objects.push(RangeVector { metric, values });
    }

    transformed_objects
        .into_iter()
        .map(RangeVector::into_series)
        .collect::<Result<Vec<_>>>()
        .and_then(|series_vector| TimeseriesVector(series_vector).to_blob())
}

fn create_graph_cell() -> Result<Vec<Cell>> {
    let graph_cell = Cell::Graph(
        GraphCell::builder()
            .id("graph".to_owned())
            .data_links(vec![format!("cell-data:{TIMESERIES_MIME_TYPE},self")])
            .graph_type(GraphType::Line)
            .stacking_type(StackingType::None)
            .build(),
    );
    Ok(vec![graph_cell])
}

fn check_status() -> Result<Blob> {
    ProviderStatus::builder()
        .status(Ok(()))
        .version(COMMIT_HASH.to_owned())
        .built_at(BUILD_TIMESTAMP.to_owned())
        .build()
        .to_blob()
}

#[pdk_export]
fn create_cells(query_type: String, _response: Blob) -> Result<Vec<Cell>> {
    log(format!("Creating cells for query type: {query_type}"));

    match query_type.as_str() {
        TIMESERIES_QUERY_TYPE => create_graph_cell(),
        _ => Err(Error::UnsupportedRequest),
    }
}

pdk_query_types! {
    TIMESERIES_QUERY_TYPE  => {
        label: "Retool: show chart",
        handler: fetch_graph_data().await,
        supported_mime_types: [TIMESERIES_MIME_TYPE]
    },
    STATUS_QUERY_TYPE => {
        handler: check_status(),
        supported_mime_types: [STATUS_MIME_TYPE]
    }
}
