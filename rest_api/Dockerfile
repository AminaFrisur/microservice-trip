FROM rust:latest
LABEL description="Rest Api of Microservice Fuhrpark"

# Set working directory
WORKDIR ./rest-api/

# create Dummy for caching dependencies in rust
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
COPY Cargo.toml .
RUN cargo install --path /rest-api

# replace dummy.rs with real src files
RUN rm -rf ./src
COPY ./src ./src

# update created and modified timestamp of main.rs
# otherwise cargo won't rebuild it.
RUN touch -a -m ./src/main.rs
RUN cargo install --path /rest-api

CMD ["restapi"]


