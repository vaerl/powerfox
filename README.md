# powerfox

Server and `systemd`-service/trigger to automatically read your power-consumption and calculate costs.

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
