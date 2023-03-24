FROM docker.io/rust:latest as build

RUN apt update
# Install firefox cause i cant be bothered to install all the other deps
RUN apt install -y mingw-w64  libssl-dev 
ADD . .
RUN cargo build --release

FROM docker.io/debian:latest as exec 
WORKDIR app/
COPY --from=build /target/release/signaling-server .
RUN chmod +x ./signaling-server

CMD ./signaling-server