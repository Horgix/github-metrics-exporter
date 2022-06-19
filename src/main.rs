use octocrab::Octocrab;
use serde::{Deserialize};
use anyhow::{Context};

mod metrics;
mod graphql_helpers;
use metrics::repository::RepoMetrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("environment variable 'GITHUB_TOKEN' is required");

    // Bootstrap an authenticated GitHub client
    let github_client = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("failed to initialize GitHub API client with Octocrab library")?;
    
    static REPOSITORY_OWNER: &'static str = "Awesome-Demo-App";
    static REPOSITORY_NAME: &'static str = "todolist-api-go";


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
          repository_owner = REPOSITORY_OWNER,
          repository_name = REPOSITORY_NAME,
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
    //println!("{:?}", repo_metrics);
    println!("{}", serde_json::to_string_pretty(&repo_metrics).unwrap());

    Ok(())
}

