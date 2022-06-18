use serde::{Serialize, Deserialize};
use super::pull_requests::PullRequestsMetrics;

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoMetrics {
    #[serde(alias = "pullRequests")]
    pull_requests: PullRequestsMetrics,
}

