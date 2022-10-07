use std::sync::Arc;
use axum::{
    Router,
    extract::{Path},
    Extension,
    routing::{get,put,delete}, http::StatusCode,
};
use tokio::sync::oneshot;

use crate::bucket::BucketActorChannel as Channel;
use crate::bucket::BucketActorMessages as Messages;
use crate::bucket::BucketMsgParams as MsgParams;
use crate::bucket::Return as Return;
use crate::bucket::BucketErrors as Errors;


async fn get_handler(Path(key): Path<String>, ext: Extension<Arc<Channel>>) -> Result<String, StatusCode> {
    let bucket = ext.0;

    let (tx, rx) = oneshot::channel::<Return>();
    let msg = Messages::Get(MsgParams{
        key: String::from(key),
        value: None,
        ch: tx,
    });
    bucket.send(msg).await.unwrap(); // XXX

    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NotFound => {
                    return Err(StatusCode::NOT_FOUND);
                },
                Errors::NoError => {
                    return Ok(r.val.unwrap());
                }
                _ => {}
            }
        },
        Err(e) => {
            // XXX
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn put_handler(body: String, ext: Extension<Arc<Channel>>) -> Result<String, StatusCode> {
    let bucket = ext.0;

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_handler(ext: Extension<Arc<Channel>>) -> Result<String, StatusCode> {
    let bucket = ext.0;

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}


pub fn mount_bucket_routes(r: Router, bucket: Channel) -> Router {
   r
       .route("/:key", get(get_handler))
       .route("/:key", put(put_handler))
       .route("/:key", delete(delete_handler))
       .layer(Extension(bucket))
}
