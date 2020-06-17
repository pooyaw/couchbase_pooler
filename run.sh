export RUST_BACKTRACE=1
export RUST_LOG=info 

export COUCHBASE_CONN=couchbase://127.0.0.1
export COUCHBASE_USER=localuser
export COUCHBASE_PASSWORD=localpass
export COUCHBASE_BUCKET=MASTER

valgrind target/debug/cb_couchbase_pooler
