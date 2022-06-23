use hyper::{
  header::CONTENT_TYPE,
  Body, Method, Request, Response,
};
use opentelemetry::{KeyValue, metrics::BatchObserverResult};
use prometheus::{TextEncoder, Encoder};
use std::sync::Arc;

use super::metrics_query;

pub async fn serve_http_requests_with_metrics_endpoint<'a>(
  req: Request<Body>,
  state: Arc<super::AppState>,
) -> Result<Response<Body>, anyhow::Error> {
  println!("Receiving request at path {}", req.uri());

  let response = match (req.method(), req.uri().path()) {
      (&Method::GET, "/metrics") => {
        for repository_name in super::REPOSITORY_NAMES {
          let repo_metrics = metrics_query::fetch_metrics_from_github(
            state.github_client.clone(),
            super::REPOSITORY_OWNER,
            repository_name
          ).await?;

          println!("{:?}", repo_metrics);
          state.opentelemetry_meter.batch_observer(move |batch| {
            let valueobserver = batch
              .u64_value_observer("pull_requests")
              .with_description("Pull Requests metrics by state")
              .init();
            move |batch_observer_result: BatchObserverResult| {
              batch_observer_result.observe(
                &[KeyValue::new("repository", format!("{}/{}", super::REPOSITORY_OWNER, repository_name))],
                //  KeyValue::new("state", "open")
                &[valueobserver.observation(repo_metrics.pull_requests.open as u64)],
              );
            }
          });
        }
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = state.exporter.registry().gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        // let result = encoder.encode_to_string(&metric_families);

        Response::builder()
            .status(200)
            .header(CONTENT_TYPE, encoder.format_type())
            .body(Body::from(buffer))
            .unwrap()
      }
      (&Method::GET, "/") => Response::builder()
          .status(200)
          .body(Body::from("Hello World!"))
          .unwrap(),
      _ => Response::builder()
          .status(404)
          .body(Body::from("Not found. Only available path are '/' and '/metrics'"))
          .unwrap(),
  };

  Ok(response)
}

