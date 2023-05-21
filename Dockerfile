FROM rust:latest as build
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/powerfox

RUN apt-get update

COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/powerfox /usr/local/bin/powerfox

EXPOSE 3000
CMD ["powerfox"]