#[macro_use] extern crate log;
extern crate env_logger;
extern crate rouille;
extern crate r2d2;
extern crate r2d2_couchbase;
extern crate serde_json;

use couchbase::{Cluster,Document,CouchbaseError};
use couchbase::document::{JsonDocument,BinaryDocument};
use futures::future::Future;
use log::Level;
use r2d2_couchbase::CouchbaseConnectionManager;
use rouille::{Request, Response, router, try_or_400, post_input};
use serde_json::{Result, Value, json};
use std::env;
use std::io;
use std::thread;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT, ATOMIC_U32_INIT};
use std::sync::{Arc, Mutex};
use std::fmt::Error;

/*
  example inserts from Django ...
  cb.upsert  bucket: MASTER   name: _origin_region_cache_key(room_slug)   value:
        { 'region: server.region, 'host': server.host }, ttl: 60 * 60 * 24 * 3

  cb.upsert  bucket: MASTER   name: username  value: { 'password' : password_hash } , ttl = 60*60*24 * 3
*/

fn reset_bucket_mem(x: &Arc<Mutex<Vec<couchbase::Bucket>>>) -> Result<()> {
    let x = x.clone(&x);
    let mut _v = x.lock().unwrap();
    while *_v.len() > 0 {
        *_v.pop()
    }
    Ok(())
}

fn malloc_bucket_handles(conn: String, user: String, password: String, bucket: String)
    -> Result<Arc<Mutex<Vec<couchbase::Bucket>>>, Error> {
    let x = Arc::new(Mutex::new(vec![]));
    for _i in 1..MAX_THREADS + 1 {
        let mut cluster = couchbase::Cluster::new(&conn).unwrap();
        cluster.authenticate(&user, &password);
        let master_bucket = cluster
            .open_bucket(&bucket, None)
            .expect("Could not open bucket");
        let handles = handles.clone(&handles);
        let mut _v = handles.lock().unwrap();
        *_v.push(master_bucket);
        info!("opened connection #{} to couchbase", _i);
    }

}


fn handle_route(request: &Request, handles: &Arc<Mutex<Vec<couchbase::Bucket>>>,
                rr_id: usize,
                b_id: u32
)
    -> Response {
    router!(request,
        (GET) (/) => {
            Response::text("CB Couchbase Pooler")
        },
        (POST) (/upsert) => {
            let data = try_or_400!(post_input!(request, {
                name: String,
                payload: String,
                ttl: u32
            }));


            let _v: Value = serde_json::from_str(&data.payload).unwrap();
            info!("\t\tdocument name: {}", data.name);
            info!("\t\tdocument payload: {}", data.payload);
            info!("\t\tdocument ttl: {}", data.ttl);
            let document : JsonDocument<Value> = JsonDocument::create(
                data.name, None, Some(data.payload.as_bytes().to_owned()), Some(data.ttl));
            if b_id == 1 {
                reset_bucket_mem(handles);
            }
            match handles[rr_id].upsert(document).wait() {
                Ok(_) => {
                    info!("\t\tdocument inserted into pool<{}>!", rr_id)
                },
                Err(_e) => {
                    error!("\t\tproblem upserting ({})", _e)
                }
            }
            Response::text("OK")
        },
        _ => {
            Response::empty_404()
        }
    )
}


const MAX_THREADS : usize = 16;
const COUCHBASE_POOL_RESET_COUNTER : u32 = 65535;


fn main() {
    env_logger::init();
    if log_enabled!(Level::Info) {
        info!("Started CB Couchbase Pooler")
    }
    let a: SocketAddr = ([0, 0, 0, 0], 8888).into();

    let conn = env::var("COUCHBASE_CONN")
        .expect("missing COUCHBASE_CONN env");
    info!("COUCHBASE_CONN loaded {}", conn);
    let user = env::var("COUCHBASE_USER")
        .expect("missing COUCHBASE_USER env");
    info!("COUCHBASE_USER loaded {}", user);
    let password = env::var("COUCHBASE_PASSWORD")
        .expect("missing COUCHBASE_PASSWORD env");
    info!("COUCHBASE_PASSWORD loaded xxx");
    let bucket = env::var("COUCHBASE_BUCKET")
        .expect("missing COUCHBASE_BUCKET env");
    info!("COUCHBASE_BUCKET loaded {}", bucket);

    let mut handles =
        malloc_bucket_handles(conn, user, password, bucket).unwrap();

    let mut handles = Arc::new(Mutex::new(vec![]));

    for _i in 1..MAX_THREADS + 1 {
            let mut cluster = couchbase::Cluster::new(&conn).unwrap();
            cluster.authenticate(&user, &password);
            let master_bucket = cluster
                .open_bucket(&bucket, None)
                .expect("Could not open bucket");
            let handles = handles.clone(&handles);
            let mut _v = handles.lock().unwrap();
            *_v.push(master_bucket);
            info!("opened connection #{} to couchbase", _i);
    }

    info!("Listening on http://0.0.0.0:8888");
    // by default, thread count is 8 x num cpus
    let rr_id : AtomicUsize = ATOMIC_USIZE_INIT;
    let b_id : AtomicU32 = ATOMIC_U32_INIT;
    rouille::start_server_with_pool(a, None, move |request| {
        rouille::log(&request, io::stdout(), || {
            let mut next_id = rr_id.fetch_add(1, Ordering::SeqCst);
            let mut next_b_id = b_id.fetch_add(1, Ordering::SeqCst);
            if next_id > MAX_THREADS - 1 {
                rr_id.store(0, Ordering::SeqCst);
                next_id = 0;
            }
            if next_b_id > COUCHBASE_POOL_RESET_COUNTER {
                b_id.store(0, Ordering::SeqCst);
                next_b_id = 0;
            }
            handle_route(&request, &handles, next_id as usize, next_b_id as u32)
        })
    });
}
