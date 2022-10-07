use std::collections::HashMap;

use murray::actor;
use tokio::sync::{oneshot, mpsc};

type Value =  String;

#[derive(Clone, Debug)]
pub enum BucketErrors {
    NoError,
    Failure,
    Conflict,
    NotFound,
}

#[derive(Clone, Debug)]
pub struct Return {
    pub val: Option<Value>,
    pub err: BucketErrors,
}

#[derive(Debug)]
pub struct BucketMsgParams {
    pub key: String,
    pub value: Option<Value>,
    pub ch: oneshot::Sender<Return>,
}


actor! {
    Bucket,
    Options: {
        id: String,
    },
    Messages: {
        Get BucketMsgParams,
        Put BucketMsgParams,
        Delete BucketMsgParams,
    },
    State: {
        store: HashMap<String, Value>,
    }
}

impl BucketActor {
    async fn handle_get(&self, state: &mut BucketActorState, msg: BucketMsgParams) {
        let mut r = Return{val: None, err: BucketErrors::NoError};
        if let Some(store) = state.store.as_mut() { 
            if let Some(v) = store.get(&msg.key) {
                r.val = Some(v.clone());
                r.err = BucketErrors::NoError; 
            } else {
                r.val = None;
                r.err = BucketErrors::NotFound;
            }
        } else { r.err = BucketErrors::Failure; };

        msg.ch.send(r).unwrap(); // XXX
    }
    
    async fn handle_put(&self, state: &mut BucketActorState, msg: BucketMsgParams) {
        let mut r = Return{val: None, err: BucketErrors::NoError};

        if let Some(store) = state.store.as_mut() { 
            if let Some(v) = msg.value {
                store.insert(msg.key, v);
            } else {
                r.err = BucketErrors::Failure;
            }
        } else { r.err = BucketErrors::Failure; };
        
        msg.ch.send(r).unwrap(); // XXX
    }
    
    async fn handle_delete(&self, state: &mut BucketActorState, msg: BucketMsgParams) {
        let mut r = Return{val: None, err: BucketErrors::NoError};

        if let Some(store) = state.store.as_mut() { 
            match store.remove(&msg.key) {
                Some(_) => {},
                None => { r.err = BucketErrors::NotFound }
            }
        } else { r.err = BucketErrors::Failure; };

        msg.ch.send(r);
    }

    fn init(&self, state: &mut BucketActorState) {
        state.store = Some(HashMap::new());
    }
}

pub fn start_bucket(name: String) -> mpsc::Sender<BucketActorMessages> {
    let actor = BucketActor{}.start(&name);

    actor
}
