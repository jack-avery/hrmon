FROM debian:bookworm-slim

COPY ./target/release/back /app/back

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install ca-certificates -y

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

WORKDIR /app
CMD ["/app/back"]
