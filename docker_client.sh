docker build -t rustclient ./client
docker run -it --name=rustclient rustclient 
docker rm client