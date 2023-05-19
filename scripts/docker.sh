# build the image
docker build -t powerfox .

# run the image
docker run --name powerfox -it powerfox