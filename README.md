# cb couchbase conn pool

## build notes

1. couchbase-rs had build issues, so it's vendored here
2. modify the /usr/lib/pkgconfig/libcouchbase.pc file to have -Cflags=-I/usr/local/include and 
copy the libcouchbase headers from /usr/include/libcouchbase into there.  if using homebrew,
modify the /usr/local/lib/pkgconfig/libcouchbase.pc file to have -Cflags=...-I/usr/local/include
4. crates use libffi libmemcached 2.10.3

## environment

1. `export COUCHBASE_CONN=couchbase://127.0.0.1,127.0.0.2,127.0.0.3`  _conn string to use_
2. `export COUCHBASE_USER=userhere` _user to authenticate against the cluster_
3. `export COUCHBASE_PASSWORD=` _password to authenticate against the cluster_
4. `export COUCHBASE_BUCKET=` _bucket to insert documents to_

