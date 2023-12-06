# random-file-server

A simple server that returns a random file inside a folder.

# `docker-compose.yml` example

```yml
services:
    random-file-server:
        image: flazepe/random-file-server
        volumes:
            - ./files:/files
        ports:
            - "8000:8000"
        restart: unless-stopped
        environment:
            PORT: 8000
```
