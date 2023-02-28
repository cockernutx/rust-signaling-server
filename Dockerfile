FROM rust:latest

RUN apt update
# Install firefox cause i cant be bothered to install all the other deps
RUN apt install -y mingw-w64  libssl-dev