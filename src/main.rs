use octocrab::Octocrab;
use serde::{Serialize, Deserialize};
use anyhow::{Context};

mod pull_requests_metrics;

#[derive(Serialize, Deserialize, Debug)]
struct RepoMetrics {
    #[serde(alias = "pullRequests")]
    pull_requests: pull_requests_metrics::PullRequestsMetrics,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("environment variable 'GITHUB_TOKEN' is required");

    // Bootstrap an authenticated GitHub client
    let github_client = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("failed to initialize GitHub API client with Octocrab library")?;

    let raw_response: serde_json::Value = github_client
        .graphql("
        query RepoStats {
            pullRequests: repository(name: \"todolist-api-go\", owner: \"Awesome-Demo-App\") {
              all: pullRequests {
                totalCount
              }
              open: pullRequests(states: OPEN) {
                totalCount
              }
              oldest: pullRequests(
                orderBy: {field: CREATED_AT, direction: ASC}
                states: OPEN
                first: 1
              ) {
                edges {
                  node {
                    createdAt
                    url
                  }
                }
              }
            }
          }")
        .await.context("failed to query GitHub GrrphQL API")?;

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
    //println!("{:?}", repo_metrics);
    println!("{}", serde_json::to_string_pretty(&repo_metrics).unwrap());

    Ok(())
}

