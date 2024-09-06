FROM docker:27.2

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev openssl-dev curl
RUN apk add --no-cache cargo=1.81.0-r0 --repository=https://dl-cdn.alpinelinux.org/alpine/edge/main
RUN cargo install --path .
RUN cp ~/.cargo/bin/container-deployer-action /usr/local/bin/container-deployer-action

ENTRYPOINT ["sh", "-c", "container-deployer-action"]
