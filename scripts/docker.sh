# build the image
docker build -t powerfox .

# run the image
docker run --name powerfox -it powerfox

# run the image with a .env-file
docker run --name powerfox -it powerfox --env-file=.env