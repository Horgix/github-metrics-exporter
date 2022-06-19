# GitHub Metrics Exporter

[![GitHub Actions badge - `Build` workflow](https://img.shields.io/github/workflow/status/Horgix/github-metrics-exporter/Build/main?label=Build)](https://github.com/Horgix/github-metrics-exporter/actions/workflows/build.yml)

This project is a "metric exporter" for GitHub metrics. Essentially, it
collects them through GitHub API (mainly the GraphQL one) and expose them e.g.
through an HTTP endpoint with a Prometheus format.

It aims at collecting metrics such as:

- Total number of PRs on a repo
- Number of open PRs on a repo
- Number of drafts PRs on a repo
- Age of the oldest open PR on a repo
- Number of PRs that wait for a given user reviews
- Number of PRs currently open by a given user
- ...

in order to create graphs, alerts, whatever you want with them!

## Status: ultra-early-alpha-wip-quickndirty-poc

This project, as you'll quickly notice by looking around, is currently more of
a test than anything else.

What that means is that **it's definitely not doing everything it claims**.

What is also means is that **contributions are more than welcome and you can
totally shape the future of this project** if you want to :)

## How it works

Summary:

- [![Octocrab
  badge](https://img.shields.io/badge/crates.io-octocrab-orange)](https://crates.io/crates/octocrab)
  `github-metrics` uses the **Octocrab** Rust GitHub client library to query
  the GitHub GraphQL API
- [![Serde
  badge](https://img.shields.io/badge/crates.io-serde-orange)](https://crates.io/crates/serde)
  The response is deserialized into proper structs using **Serde** and **Serde
  JSON** with as much default (de)serializer as possible, but also some custom
  ones (e.g. to extract nested values from GraphQL reponses or to compute
  `Durations` between a given date and now)
- [![Anyhow
  badge](https://img.shields.io/badge/crates.io-anyhow-orange)](https://crates.io/crates/anyhow)
  **Anyhow** is used to give proper context to errors while still keeping a
  readable code
