FROM rust:1.82-bookworm AS build

WORKDIR /app
COPY frontend ./frontend
COPY spin-meili ./spin-meili
COPY fixtures/movies.json ./fixtures/movies.json

RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && cd frontend && npm ci && npm run build \
    && cd .. \
    && cargo build --manifest-path spin-meili/Cargo.toml --release -p oci-http-adapter

FROM debian:bookworm-slim

COPY --from=build /app/spin-meili/target/release/oci-http-adapter /usr/local/bin/movie-search

EXPOSE 7700
ENV MOVIE_SEARCH_ADDR=0.0.0.0:7700
CMD ["movie-search"]
