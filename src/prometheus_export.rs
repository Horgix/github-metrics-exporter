use hyper::{
  header::CONTENT_TYPE,
  Body, Method, Request, Response,
};
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
          let repo_metrics = metrics_query::fetch_metrics_from_github(
            &state.github_client,
            super::REPOSITORY_OWNER,
            super::REPOSITORY_NAME,
          ).await?;
          println!("{:?}", repo_metrics);
          //state.pull_requests_gauge.(repo_metrics.pull_requests.open.into(),
          //&[KeyValue::new(
          //  "repository", format!("{repository_owner}/{repository_name}",
          //                        repository_owner=REPOSITORY_OWNER,
          //                        repository_name=repo)
          //  ),
          //  KeyValue::new("state", "open")
          //]);

          let mut buffer = vec![];
          let encoder = TextEncoder::new();
          let metric_families = state.exporter.registry().gather();
          encoder.encode(&metric_families, &mut buffer).unwrap();

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

