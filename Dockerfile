FROM rust:latest as build
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/powerfox

COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/static-debian11:debug

COPY --from=build /usr/local/cargo/bin/powerfox /usr/local/bin/powerfox

EXPOSE 3000
CMD ["powerfox"]