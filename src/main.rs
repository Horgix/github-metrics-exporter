use octocrab::Octocrab;
use serde::{de,Serialize, Deserialize, Deserializer};
use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Debug)]
struct PullRequestsMetrics {
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    all: i32,
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    open: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct RepoMetrics {
    #[serde(alias = "pullRequests")]
    pull_requests: PullRequestsMetrics,
}

fn deserialize_nested_total_count<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    #[derive(Serialize, Deserialize, Debug)]
    struct TotalCount {
        #[serde(alias = "totalCount")] 
        total_count: i32,
    }

    let res: TotalCount = de::Deserialize::deserialize(deserializer)?;
    return Ok(res.total_count);
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
            pullRequests: repository(name: \"todolistqwe-api-go\", owner: \"Awesome-Demo-App\") {
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

    // if response.contains_key("errors") {
    //     eprintln!("GraphQL query returned errors: {:?}", response.get("errors"));
    //     Err(())
    // }
    let repo_metrics: RepoMetrics = Deserialize::deserialize(
            response
            .get("data")
            .context("failed to find 'data' key inside response")?
        ).context("failed to deserialize GraphQL query answer")?;
    println!("{:?}", repo_metrics);

    Ok(())
}

