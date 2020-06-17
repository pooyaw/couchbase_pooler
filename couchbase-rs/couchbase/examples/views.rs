extern crate couchbase;
extern crate futures;

use couchbase::{Cluster, ViewQuery, ViewResult};
use futures::Stream;

/// Opens a bucket and then runs a view query and prints out the results.
fn main() {
    // Initialize the Cluster
    let cluster = Cluster::new("localhost").expect("Could not initialize Cluster");

    // Open the travel-sample bucket
    let bucket = cluster
        .open_bucket("beer-sample", None)
        .expect("Could not open Bucket");

    // Run the query and iterate the rows.
    for row in bucket
        .query_view(ViewQuery::from("beer", "brewery_beers").limit(3))
        .wait()
    {
        match row {
            Ok(ViewResult::Row(r)) => println!("Found Row {:?}", r),
            Ok(ViewResult::Meta(m)) => println!("Found Meta {:?}", m),
            Err(e) => panic!("Error! {:?}", e),
        }
    }
}
