# powerfox

Server and `systemd`-service/trigger to automatically read your power-consumption and calculate costs.

## Setup

Make sure that the database contains config-values.
See [this file](./sql/initial-config.sql) for more information.

This could be implemented with a bot-command, but this seems annoying.

## Database-Access

`sqlx` requires a super-user [to work properly](https://github.com/launchbadge/sqlx/discussions/2051).
It also needs the environment-variable `DATABASE_URL` to work.

## Triggering summaries

Summaries are now triggered using [`tokio-cron-scheduler`](https://crates.io/crates/tokio-cron-scheduler).

## Networking

This container needs access to a database.
Both this server and database are running with Docker (- I've opted against using `docker compose` for now).

To make the database-container accessible from the server-container, follow these steps:

```shell
# create a new network
docker network create <network>

# add the database-container to that network
docker network connect <network> <database>

# add the server-container to that network
docker network connect <network> <server>
```

**This enables each container in that network to communicate with other containers by using their container-name as the hostname.**
