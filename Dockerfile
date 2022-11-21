# Client and renderer
FROM rust:1.63.0 AS wasm_build_base

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | bash

WORKDIR /build

RUN mkdir client && \
    mkdir common && \
    mkdir renderer

COPY renderer/Cargo.toml renderer
COPY renderer/Cargo.lock renderer
COPY client/Cargo.toml client
COPY client/Cargo.lock client
COPY common/Cargo.toml common
COPY common/Cargo.lock common

RUN cd common && \
    mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cd ../client && \
    mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cargo build && \
    cd ../renderer && \
    mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cargo build

RUN cd client && \
    wasm-pack build

FROM wasm_build_base AS client_build

WORKDIR /build

RUN rm -rf client/src && \
    rm -rf common/src

COPY client ./client
COPY common ./common

RUN mkdir /output && \
    touch common/src/lib.rs && \
    cd client && \
    touch src/lib.rs && \
    # For error visibility
    cargo check && \
    wasm-pack build --verbose --target web --debug --out-dir /output

FROM wasm_build_base AS renderer_build

WORKDIR /build

RUN rm -rf renderer/src && \
    rm -rf common/src

COPY renderer ./renderer
COPY common ./common

RUN mkdir /output && \
    touch common/src/lib.rs && \
    cd renderer && \
    touch src/lib.rs && \
    # For error visibility
    cargo check && \
    wasm-pack build --verbose --target web --debug --out-dir /output

# Front
FROM denoland/deno:debian AS front_build_base

WORKDIR /build

COPY front/package.json .
COPY front/package-lock.json .

RUN apt-get update && \
    apt-get install npm -y && \
    npm i

FROM front_build_base AS front_build

WORKDIR /build

COPY front/* ./

RUN mkdir built

COPY --from=client_build /output/* built/

RUN deno run \
        --allow-run \
        --allow-env \
        --allow-read \
        --allow-write \
        --allow-net \
        build.ts && \
    deno compile \
        --allow-env \
        --allow-net \
        --allow-read \
        server.ts && \
    mkdir /output && \
    mkdir /output/built && \
    cp built/* /output/built && \
    cp index.html /output && \
    cp server /output

# Server
FROM rust:1.63.0 AS server_build_base

WORKDIR /build

RUN mkdir server && \
    mkdir common

COPY server/Cargo.toml server
COPY server/Cargo.lock server
COPY common/Cargo.toml common
COPY common/Cargo.lock common

RUN cd common && \
    mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cd ../server && \
    mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cargo build

FROM server_build_base AS server_build

WORKDIR /build

RUN rm -rf server/src && \
    rm -rf common/src

COPY server ./server
COPY common ./common

RUN mkdir /output && \
    touch common/src/lib.rs && \
    cd server && \
    cargo build && \
    cp target/debug/server /output

# Runtime
FROM debian:bullseye-slim AS runtime_base

EXPOSE 80

RUN apt-get update && \
    apt-get install nginx -y

FROM runtime_base AS runtime

COPY forward/combined.conf /etc/nginx/conf.d/default.conf
COPY container_entry.sh /entry.sh
COPY --from=front_build /output /front-server
COPY --from=server_build /output /server

ENTRYPOINT ["/bin/bash", "./entry.sh"]
