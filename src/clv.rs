use fiberplane_pdk::prelude::Timestamp;
use fiberplane_pdk::providers::*;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    num::ParseFloatError,
    time::{Duration, SystemTime},
};

#[derive(Deserialize)]
pub struct PrometheusPoint(pub f64, pub String);

impl PrometheusPoint {
    pub fn to_metric(&self) -> Result<Metric, ParseFloatError> {
        let time = SystemTime::UNIX_EPOCH + Duration::from_millis((self.0 * 1000.0) as u64);
        Ok(Metric::builder()
            .time(Timestamp::from(time))
            .value(self.1.parse()?)
            .otel(OtelMetadata::default())
            .build())
    }
}

#[derive(Deserialize)]
pub struct RangeVector {
    pub metric: BTreeMap<String, String>,
    pub values: Vec<PrometheusPoint>,
}

impl RangeVector {
    pub fn into_series(self) -> Result<Timeseries, Error> {
        let mut labels = self.metric;
        let name = labels.remove("__name__").unwrap_or_else(|| "".to_owned());
        let metrics = self
            .values
            .into_iter()
            .map(|value| value.to_metric())
            .collect::<Result<_, _>>()?;
        Ok(Timeseries::builder()
            .name(name)
            .labels(labels)
            .metrics(metrics)
            .otel(OtelMetadata::default())
            .visible(true)
            .build())
    }
}
