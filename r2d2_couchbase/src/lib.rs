extern crate couchbase;
extern crate r2d2;

use std::error;
use std::error::Error as _StdError;
use std::fmt;
use couchbase::{Cluster, Document, CouchbaseError, Bucket};
use couchbase::document::BinaryDocument;

/// A unified enum of errors returned by couchbase::Cluster
#[derive(Debug)]
pub enum Error {
    Other(couchbase::CouchbaseError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.description(), self.cause().unwrap())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Other(ref err) => err.cause()
        }
    }
}

/// ## Example
///
/// ```rust
/// extern crate r2d2;
/// extern crate r2d2_couchbase;
/// extern crate serde_json;
///
/// use r2d2_couchbase::{CouchbaseConnectionManager};
///
/// use std::thread;
///
/// fn main() {
///     let config = r2d2::Config::default();
///     let manager = CouchbaseConnectionManager::new("couchbase://localhost/").unwrap();
///     let pool = r2d2::Pool::new(config, manager).unwrap();
///
///     let mut handles = vec![];
///
///     for i in 0..20 {
///         let pool = pool.clone();
///         handles.push(thread::spawn(move || {
///             let content = serde_json::builder::ObjectBuilder::new()
///                 .insert("foo", i)
///                 .unwrap();
///             println!("Sending {}", &content);
///             let conn = pool.get().unwrap();
///             conn.create_document("/test", &content).run().unwrap();
///         }));
///     }
///
///     for handle in handles {
///         handle.join().unwrap()
///     }
/// }
/// ```
#[derive(Debug)]
pub struct CouchbaseConnectionManager {
    server_url: String,
    username: String,
    password: String,
    bucket: String,
}

impl CouchbaseConnectionManager {
    pub fn new(server_url: &str, username: &str, password: &str, bucket: &str)
            -> Result<CouchbaseConnectionManager, couchbase::CouchbaseError> {
        Ok(CouchbaseConnectionManager {
            server_url: server_url.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
            bucket: bucket.to_owned(),
        })
    }
}

impl r2d2::ManageConnection for CouchbaseConnectionManager {
    type Connection = Bucket;
    type Error = Error;

    fn connect(&self) -> Result<Bucket, Error> {
        match couchbase::Cluster::new(&self.server_url) {
            Ok(mut c) => {
                c.authenticate(&self.username, &self.password);
                c.open_bucket(&self.bucket, None)
                    .map_err(|e| Error::Other(e))
            }
            Err(E) => Err(Error::Other(E))
        }

    }

    fn is_valid(&self, _conn: &mut Bucket) -> Result<(), Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Bucket) -> bool {
        false
    }
}
