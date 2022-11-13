FROM rust:1.65 AS labels

LABEL org.opencontainers.image.title="Distributed Arcade"
LABEL org.opencontainers.image.description="Fast and simple scoreboard service for games"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.url="https://github.com/Steffo99/distributed-arcade"
LABEL org.opencontainers.image.authors="Stefano Pigozzi <me@steffo.eu>"


FROM labels AS files

WORKDIR /usr/src/distributed_arcade
COPY . .


FROM files AS install

RUN cargo install --path . --all-features --bins


FROM install AS environment

ENV RUST_LOG "warn,distributed_arcade=info"
ENTRYPOINT ["distributed_arcade"]
