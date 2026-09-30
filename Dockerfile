FROM node:22-slim AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1-slim AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY proxy/Cargo.toml proxy/Cargo.toml
COPY proxy/src proxy/src
COPY proxy/static proxy/static
COPY --from=frontend /app/frontend/dist/index.html proxy/static/index.html
COPY --from=frontend /app/frontend/dist/assets proxy/static/assets
RUN cargo build --release -p canicule

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home canicule \
    && mkdir -p /var/cache/canicule \
    && chown canicule:canicule /var/cache/canicule
COPY --from=backend /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=backend /app/target/release/canicule /usr/local/bin/canicule
USER canicule
ENV PORT=8080 CACHE_DIR=/var/cache/canicule
EXPOSE 8080
VOLUME /var/cache/canicule
CMD ["canicule"]
