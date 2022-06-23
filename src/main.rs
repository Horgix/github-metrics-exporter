use octocrab::Octocrab;
use anyhow::{Context};
// use futures::future;

//  use opentelemetry::{KeyValue, metrics::BatchObserverResult};
use opentelemetry_prometheus::PrometheusExporter;

mod metrics;
mod graphql_helpers;
mod metrics_query;
mod prometheus_export;

use hyper::{
  service::{make_service_fn, service_fn},
  Server,
};
use std::sync::Arc;

pub static REPOSITORY_OWNER: &'static str = "Horgix";
pub static REPOSITORY_NAME: &'static str = "todolist-api-go";
pub static REPOSITORY_NAMES: &'static [&'static str] = &[
  "incidents-automation-app",
  "kind-demo",
  "github-demo",
];

pub struct AppState {
  exporter: PrometheusExporter,
  github_client: octocrab::Octocrab,
  // pull_requests_gauge: opentelemetry::metrics::ValueObserver<u64>,
  //metrics_observer: BatchObserverResult,
  opentelemetry_meter: opentelemetry::metrics::Meter,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let token = std::env::var("GITHUB_TOKEN").expect("environment variable 'GITHUB_TOKEN' is required");
    // Bootstrap an authenticated GitHub client
    let github_client = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("failed to initialize GitHub API client with Octocrab library")?;
    
    // let repo_metrics = fetch_metrics_from_github(github_client, REPOSITORY_OWNER, REPOSITORY_NAME).await?;

    //println!("{:?}", repo_metrics);
    //println!("{}", serde_json::to_string_pretty(&repo_metrics).unwrap());

    let exporter = opentelemetry_prometheus::exporter().init();

    // Metrics for OpenTelemetry
    let meter = opentelemetry::global::meter("github-metrics-exporter");

    // let one_metric_callback =
    //   async move |res: ObserverResult<u64>| {
    //       let metrics = fetch_metrics_from_github(
    //         &github_client,
    //         REPOSITORY_OWNER,
    //         REPOSITORY_NAME
    //       );
    //       match metrics {
    //         Ok(m) => res.observe(m.repositories.all_pull_requests, &[]),

    //       }
    //       //metrics.repositories.;
    //       //res.observe({
    //   };
          //state.pull_requests_gauge.(repo_metrics.pull_requests.open.into(),
          //&[KeyValue::new(
          //  "repository", format!("{repository_owner}/{repository_name}",
          //                        repository_owner=REPOSITORY_OWNER,
          //                        repository_name=repo)
          //  ),
          //  KeyValue::new("state", "open")
          //]);
  // let newrecorder = meter
  //     .u64_value_observer("ex.com.one", one_metric_callback)
  //     .with_description("A ValueObserver set to 1.0")
  //     .init();
    
    let state = Arc::new(AppState {exporter, github_client: github_client.clone(), opentelemetry_meter: meter});//, github_client});//, pull_requests_gauge: newrecorder});

    let make_svc = make_service_fn(move |_conn| {
      let state = state.clone();

      // This is the `Service` that will handle the connection.
      // `service_fn` is a helper to convert a function that
      // returns a Response into a `Service`.
      async move {
        Ok::<_, std::convert::Infallible>(
          service_fn(
            move |req| prometheus_export::serve_http_requests_with_metrics_endpoint(req, state.clone())
          )
        )
      }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;
      
    Ok(())
  }

