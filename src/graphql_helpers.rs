pub static GRAPHQL_REPO_SUBQUERY_ALL_PULL_REQUESTS: &'static str = "
  all: pullRequests {
    totalCount
  }
";

pub static GRAPHQL_REPO_SUBQUERY_ALL_OPEN_PULL_REQUEST: &'static str = "
  open: pullRequests(states: OPEN) {
    totalCount
  }
";

pub static GRAPHQL_REPO_SUBERY_OLDEST_PULL_REQUEST: &'static str = "
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
";