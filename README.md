# powerfox

Server and `systemd`-service/trigger to automatically read your power-consumption and calculate costs.

## Triggering summaries

Summaries are triggered by a request to the according endpoint - right now, these summaries are supported:

```http
# daily summary
GET http://gojo:3000/powerfox/daily
```

This request is done by a `systemd`-[service](scripts/powerfox-daily.service) which gets triggered by a [timer](scripts/powerfox-daily.timer).
Follow these steps to add a new timer-service-combo:

1. create both timer and service
2. symlink timer and service to `/etc/systemd/system`
3. `start` and `enable` the timer.

Commands for these steps can be found [here](scripts/systemd.sh).

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
