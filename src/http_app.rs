use std::{sync::Arc};
use axum::{
    Router,
    extract::{Path},
    body::Bytes,
    Extension,
    routing::{get,post,put,delete}, http::StatusCode,
};
use tokio::sync::oneshot;

use crate::supervisor::BidonActorChannel as Channel;
use crate::supervisor::BidonActorMessages as Messages;
use crate::supervisor::MsgParams as MsgParams;
use crate::bucket::Return as Return;
use crate::bucket::BucketErrors as Errors;

async fn createns_handler(Path(ns): Path<String>, ext: Extension<Arc<Channel>>) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();
    let (tx, rx) = oneshot::channel::<Return>();
    let msg = Messages::CreateNS(MsgParams{
        ns: String::from(ns),
        key: None,
        value: None,
        ch: tx,
    });
    bidon.send(msg).await.unwrap(); // XXX

    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::Conflict => {
                    return Err(StatusCode::CONFLICT);
                },
                Errors::NoError => {
                    return Err(StatusCode::CREATED);
                }
                _ => {}
            }
        },
        Err(_e) => {
            // XXX
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn loadns_handler(Path(ns): Path<String>, ext: Extension<Arc<Channel>>) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn savens_handler(Path(ns): Path<String>, ext: Extension<Arc<Channel>>) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();
    
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_handler(Path((ns, key)): Path<(String, String)>, ext: Extension<Arc<Channel>>) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();

    let (tx, rx) = oneshot::channel::<Return>();
    let msg = Messages::Get(MsgParams{
        ns: String::from(ns),
        key: Some(String::from(key)),
        value: None,
        ch: tx,
    });
    bidon.send(msg).await.unwrap(); // XXX

    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NotFound => {
                    return Err(StatusCode::NOT_FOUND);
                },
                Errors::NoError => {
                    return Ok(Bytes::from(r.val.unwrap()));
                }
                _ => {}
            }
        },
        Err(_e) => {
            // XXX
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn put_handler(Path((ns, key)): Path<(String, String)>, ext: Extension<Arc<Channel>>, body: Bytes) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();
    let (tx, rx) = oneshot::channel::<Return>();
    let msg = Messages::Put(MsgParams{
        ns: String::from(ns),
        key: Some(String::from(key)),
        value: Some(body.to_vec()),
        ch: tx,
    });
    bidon.send(msg).await.unwrap(); // XXX

    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NotFound => {
                    return Err(StatusCode::NOT_FOUND);
                },
                Errors::NoError => {
                    let prev = if let Some(prev) = r.val { Bytes::from(prev) } else { Bytes::new() };
                    return Ok(prev);
                }
                _ => {}
            }
        },
        Err(_e) => {
            // XXX
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_handler(Path((ns, key)): Path<(String, String)>, ext: Extension<Arc<Channel>>) -> Result<Bytes, StatusCode> {
    let bidon = ext.0.clone();
    let (tx, rx) = oneshot::channel::<Return>();
    let msg = Messages::Delete(MsgParams{
        ns: String::from(ns),
        key: Some(String::from(key)),
        value: None,
        ch: tx,
    });
    bidon.send(msg).await.unwrap(); // XXX

    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NoError => {
                    let prev = if let Some(prev) = r.val { Bytes::from(prev) } else { Bytes::new() };
                    return Ok(prev);
                }
                _ => {}
            }
        },
        Err(_e) => {
            // XXX
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}


pub fn mount_bidon_routes(r: Router, bidon: Channel) -> Router {
   r
       .route("/:ns", post(createns_handler))
       .route("/:ns/_load", post(loadns_handler))
       .route("/:ns/_save", post(savens_handler))
       .route("/:ns/:key", get(get_handler))
       .route("/:ns/:key", put(put_handler))
       .route("/:ns/:key", delete(delete_handler))
       .layer(Extension(Arc::new(bidon)))
}

#[cfg(test)]
mod tests {
    use hyper::{Request,Body, StatusCode};
    use axum::{Router};
    use hyper_mock_client::MockClient;

    #[tokio::test]
    async fn test_ns_create() {
        
       let router = Router::new(); 
    let bidon = crate::supervisor::start_bidon().await;
    let app = crate::http_app::mount_bidon_routes(router, bidon);
    let mut client = MockClient::new(app);

    let req = Request::builder().uri("http://localhost:3000/default/foo").body(Body::empty()).unwrap();
    assert_eq!(client.request(req).await.status(), StatusCode::NOT_FOUND);
    

    assert_eq!(true, false);

    }
}
