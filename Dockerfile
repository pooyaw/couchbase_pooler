FROM rust:latest 

WORKDIR /usr/src/app

COPY ./ ./
ENV RUST_BACKTRACE 1
ENV RUST_LOG info

RUN apt-get update && apt-get -y install cmake libuv1 libuv1-dev openjdk-8-jre-headless

RUN cargo install bindgen 
RUN cargo build --release
RUN cargo install --path .

RUN mkdir /usr/local/cargo/lib
RUN find /usr/src/app/target/release/build -iname 'libcouchbase*.so' -exec cp {} /usr/local/cargo/lib/ \;
RUN cp /usr/local/cargo/lib/libcouchbase.so /usr/local/cargo/lib/libcouchbase.so.2

ENV LD_LIBRARY_PATH /usr/local/cargo/lib
CMD ["/usr/local/cargo/bin/cb_couchbase_pooler"]
