docker build -t rustclient ./client
docker rm rustclient
docker run -it --name=rustclient rustclient 

