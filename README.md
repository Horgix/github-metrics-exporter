# GitHub Metrics Exporter

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
