FROM rust:1.65 AS files
WORKDIR /usr/src/revenants-brooch
COPY . .

FROM files AS install
RUN cargo install --path . --all-features --bins

FROM debian:buster AS system
RUN apt-get update
RUN apt-get install -y libssl1.1 ca-certificates
RUN rm -rf /var/lib/apt/lists/*
COPY --from=install /usr/local/cargo/bin/revenants_brooch /usr/local/bin/revenants_brooch

FROM system AS entrypoint
ENTRYPOINT ["distributed_arcade"]
CMD []

FROM entrypoint AS final
LABEL org.opencontainers.image.title="Distributed Arcade"
LABEL org.opencontainers.image.description="Fast and simple scoreboard service for games"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.url="https://github.com/Steffo99/distributed-arcade"
LABEL org.opencontainers.image.authors="Stefano Pigozzi <me@steffo.eu>"
ENV RUST_LOG "warn,distributed_arcade=info"
