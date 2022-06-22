use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use anyhow::{Result};
use chrono::{DateTime, FixedOffset, Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct PullRequestsMetrics {
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    pub all: i32,
    #[serde(deserialize_with = "deserialize_nested_total_count")]
    pub open: i32,
    #[serde(deserialize_with = "deserialize_duration_since_creation_date")]
    #[serde(serialize_with = "serialize_duration")]
    pub oldest: Duration,
}
fn deserialize_nested_total_count<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    #[derive(Serialize, Deserialize, Debug)]
    struct TotalCount {
        #[serde(alias = "totalCount")] 
        total_count: i32,
    }

    let res: TotalCount = de::Deserialize::deserialize(deserializer)?;
    Ok(res.total_count)
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
        panic!("Didn't find any edge - this should be handled gracefully");
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
