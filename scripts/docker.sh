# build the image
docker build -t powerfox .

# build the image without any caching - caching caused some confusion early on
docker build -t powerfox . --no-cache

# run the image with the applicable environment
docker run --name powerfox -d --env-file ./.env --network=base powerfox

# run bash inside the container
docker exec -it powerfox bash