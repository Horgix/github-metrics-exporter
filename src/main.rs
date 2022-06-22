use octocrab::Octocrab;
use serde::{Deserialize};
use anyhow::{Context};
// use futures::future;

// use opentelemetry::{KeyValue, metrics::ObserverResult};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{TextEncoder, Encoder};

mod metrics;
mod graphql_helpers;
use metrics::repository::RepoMetrics;

use hyper::{
  header::CONTENT_TYPE,
  service::{make_service_fn, service_fn},
  Body, Method, Request, Response, Server,
};
use std::sync::Arc;

static REPOSITORY_OWNER: &'static str = "Awesome-Demo-App";
static REPOSITORY_NAME: &'static str = "todolist-api-go";

async fn serve_req<'a>(
  req: Request<Body>,
  state: Arc<AppState>,
) -> Result<Response<Body>, anyhow::Error> {
  println!("Receiving request at path {}", req.uri());

  let response = match (req.method(), req.uri().path()) {
      (&Method::GET, "/metrics") => {
          let repo_metrics = fetch_metrics_from_github(
            &state.github_client,
            REPOSITORY_OWNER,
            REPOSITORY_NAME,
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
          .body(Body::from("Hello World"))
          .unwrap(),
      _ => Response::builder()
          .status(404)
          .body(Body::from("Missing Page"))
          .unwrap(),
  };

  Ok(response)
}

struct AppState {
  exporter: PrometheusExporter,
  github_client: octocrab::Octocrab,
  // pull_requests_gauge: opentelemetry::metrics::ValueObserver<u64>,
}

async fn fetch_metrics_from_github(
  github_client: &octocrab::Octocrab,
  repository_owner: &str,
  repository_name: &str,
) -> Result<RepoMetrics, anyhow::Error> {
    let raw_response: serde_json::Value = github_client
        .graphql(&format!(
        "
        query RepoStats {{
            pullRequests: repository(owner: \"{repository_owner}\", name: \"{repository_name}\") {{
                {all_pull_requests}
                {open_pull_requests}
                {older_pull_request}
            }}
          }}
          ",
          repository_owner = repository_owner,
          repository_name = repository_name,
          all_pull_requests=graphql_helpers::GRAPHQL_REPO_SUBQUERY_ALL_PULL_REQUESTS,
          open_pull_requests=graphql_helpers::GRAPHQL_REPO_SUBQUERY_ALL_OPEN_PULL_REQUEST,
          older_pull_request=graphql_helpers::GRAPHQL_REPO_SUBERY_OLDEST_PULL_REQUEST,
        ))
        .await.context("failed to query GitHub GraphQL API")?;

    let response = raw_response.as_object().context("failed to interpret GitHub GraphQL API answer as a Map")?;

    /*
    If, for example, the GraphQL query targets an unexisting repository,
    the answer will not be a total error. Instead, it will include an `errors` key.
    Example:
        {
          "data": {
            "pullRequests": null
          },
          "errors": [
            {
              "type": "NOT_FOUND",
              "path": [
                "pullRequests"
              ],
              "locations": [
                {
                  "line": 7,
                  "column": 13
                }
              ],
              "message": "Could not resolve to a Repository with the name 'UnexistingOwner/UnexistingRepo'."
            }
          ]
        }
    The following condition handles that case, though the errors won't be formatted nicely
    */
    if response.contains_key("errors") {
        anyhow::bail!(format!("found errors in the GraphQL API query answer: {:?}", response.get("errors")))
    }
    let repo_metrics: RepoMetrics = Deserialize::deserialize(
            response
            .get("data")
            .context("failed to find 'data' key inside response")?
        ).context("failed to deserialize GraphQL query answer")?;

    Ok(repo_metrics)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("environment variable 'GITHUB_TOKEN' is required");

    // Bootstrap an authenticated GitHub client
    let github_client = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("failed to initialize GitHub API client with Octocrab library")?;
    
    // static REPOSITORY_OWNER: &'static str = "Awesome-Demo-App";
    // static REPOSITORY_NAME: &'static str = "todolist-api-go";
// 
    // let repo_metrics = fetch_metrics_from_github(github_client, REPOSITORY_OWNER, REPOSITORY_NAME).await?;

    //println!("{:?}", repo_metrics);
    //println!("{}", serde_json::to_string_pretty(&repo_metrics).unwrap());

    let exporter = opentelemetry_prometheus::exporter().init();

    // Metrics for OpenTelemetry
    // let meter = opentelemetry::global::meter("github-metrics-exporter");

    // let recorder = meter
    //   .i64_up_down_counter("pull_requests")
    //   .with_description("Pull Requests metrics by state")
    //   .init();

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
    
    let state = Arc::new(AppState {exporter, github_client});//, pull_requests_gauge: newrecorder});


    // let encoder = TextEncoder::new();
    // let metric_families = exporter.registry().gather();
    // println!("{:?}", metric_families);
    // let result = encoder.encode_to_string(&metric_families);

    // println!("{}", result.unwrap());

    let make_svc = make_service_fn(move |_conn| {
      let state = state.clone();
      // This is the `Service` that will handle the connection.
      // `service_fn` is a helper to convert a function that
      // returns a Response into a `Service`.
      async move { Ok::<_, std::convert::Infallible>(service_fn(move |req| serve_req(req, state.clone()))) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;
      
    Ok(())
  }

