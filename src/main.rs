use octocrab::Octocrab;
use serde::{de,Serialize, Deserialize, Deserializer};

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
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    // Bootstrap an authenticated GitHub client
    let github_client = Octocrab::builder().personal_token(token).build()?;

    let response: serde_json::Value = github_client
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
        .await?;

    match response.as_object() {
        Some(map) => match map.get("data") {
            Some(map) => {
                match Deserialize::deserialize(map) {
                    Ok::<RepoMetrics, _>(repometrics) => println!("{:?}", repometrics),
                    Err(_) => println!("LOL FAILED3"),
                }
            },
            None => println!("LOL FAILED1"),
        }
        None => println!("LOL FAILED2"),
    }

    Ok(())
}

