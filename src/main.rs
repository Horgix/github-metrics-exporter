use octocrab::Octocrab;
use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Duration};

#[derive(Serialize, Deserialize, Debug)]
struct PullRequestsMetrics {
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    all: i32,
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    open: i32,
    #[serde(deserialize_with = "deserialize_duration_since_creation_date")]
    #[serde(serialize_with = "serialize_duration")]
    oldest: Duration,
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

// Deserializes a nested `createdAt` info from a GitHub GraphQL query response
// into a Duration since this time
// Example of input:
//  "oldest": {
//    "edges": [
//      {
//        "node": {
//          "createdAt": "2021-12-31T25:59:59Z",
//          "url": "https://github.com/SomeOwner/SomeRepo/pull/42"
//        }
//      }
//    ]
//  }
fn deserialize_duration_since_creation_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    let answer: serde_json::Map<String, serde_json::Value> = de::Deserialize::deserialize(deserializer)?;
    let edges = answer
        .get("edges")
        .expect("could not find `edges`")
        .as_array()
        .expect("could not interpret `edges` as an array");

    if edges.len() != 1 {
        // TODO
    }
    let raw_created_at: &str = edges[0]
        .as_object().expect("failed to interpret first edge as a Map")
        .get("node").expect("failed to find node in first edge")
        .as_object().expect("failed to intepret node of first edge as a Map")
        .get("createdAt").expect("failed to find `createdAt` in node")
        .as_str().expect("failed to interpret `createdAt` as a String");

    println!("{:?}", raw_created_at);

    let created_at: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(raw_created_at).unwrap();
    println!("{:?}", created_at);
    let duration_since_created_at: Duration = created_at.signed_duration_since(chrono::Utc::now());
    Ok(duration_since_created_at)
}

pub fn serialize_duration<S>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{} days", duration.num_days().abs());
    serializer.serialize_str(&s)
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
    println!("{:?}", repo_metrics);

    Ok(())
}

