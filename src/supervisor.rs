use std::collections::HashMap;
use murray::actor;
use tokio::sync::{oneshot, mpsc};


use crate::bucket::BucketActor as Bucket;
use crate::bucket::BucketActorChannel as BucketChannel;
use crate::bucket::BucketActorMessages as BucketMessages;
use crate::bucket::BucketMsgParams as BucketMsgParams;
use crate::bucket::Return as Return;
use crate::bucket::BucketErrors as Errors;
use crate::bucket::Value as Value;

type BucketsMap = HashMap<String, BucketChannel>;

#[derive(Debug)]
pub struct MsgParams {
    pub ns: String,
    pub key: Option<String>,
    pub value: Option<Value>,
    pub ch: oneshot::Sender<Return>,
}
    

actor! {
    Bidon,
    Messages: {
        CreateNS MsgParams,
        SaveNS MsgParams,
        LoadNS MsgParams,
        Get MsgParams,
        Put MsgParams,
        Delete MsgParams,
    },
    State: {
        buckets: BucketsMap, 
    },
}

async fn route_to_bucket(msg: BidonActorMessages, buckets: &BucketsMap) {
    let ns: String;

    let bucketmsg = match msg {
        BidonActorMessages::Get(msg) => {
            ns = msg.ns;
            BucketMessages::Get(BucketMsgParams{key: msg.key.unwrap(), value: msg.value, ch: msg.ch})
        },
        BidonActorMessages::Delete(msg) => {
            ns = msg.ns;
            BucketMessages::Delete(BucketMsgParams{key: msg.key.unwrap(), value: msg.value, ch: msg.ch})
        },
        BidonActorMessages::Put(msg) => {
            ns = msg.ns;
            BucketMessages::Put(BucketMsgParams{key: msg.key.unwrap(), value: msg.value, ch: msg.ch})
        },
        _ => { todo!() },
    };

    match buckets.get(&ns) {
        Some(bucket) => {
            bucket.send(bucketmsg).await.unwrap()
        },
        None => {
            let retch = match bucketmsg {
                BucketMessages::Get(m) => { m.ch },
                BucketMessages::Put(m) => { m.ch },
                BucketMessages::Delete(m) => { m.ch },
            };
            retch.send(Return{val: None, err: Errors::NotFound}).unwrap()
        }
    };
}

impl BidonActor {

    fn init(&self, state: &mut BidonActorState) {
        state.buckets = Some(HashMap::new());
    }

    async fn handle_createns(&self, state: &mut BidonActorState, msg: MsgParams) {
        let mut r = Return{val: None, err: Errors::NoError};
        let buckets = state.buckets.as_mut().unwrap();
        if buckets.contains_key(&msg.ns) {
            r.err = Errors::Conflict;
        } else {
            let bucket = Bucket{}.start(&msg.ns);
            buckets.insert(msg.ns, bucket);
        }
        msg.ch.send(r).unwrap();
    }
    
    async fn handle_savens(&self, state: &mut BidonActorState, msg: MsgParams) {
        let mut r = Return{val: None, err: Errors::Failure};

        msg.ch.send(r).unwrap();
        
    }
    async fn handle_loadns(&self, state: &mut BidonActorState, msg: MsgParams) {
        let mut r = Return{val: None, err: Errors::NotFound};
        
        msg.ch.send(r).unwrap();
    }
    async fn handle_get(&self, state: &mut BidonActorState, msg: MsgParams) {
        route_to_bucket(BidonActorMessages::Get(msg), state.buckets.as_ref().unwrap()).await 
    }
    async fn handle_put(&self, state: &mut BidonActorState, msg: MsgParams) {
        route_to_bucket(BidonActorMessages::Put(msg), state.buckets.as_ref().unwrap()).await 
    }
    async fn handle_delete(&self, state: &mut BidonActorState, msg: MsgParams) {
        route_to_bucket(BidonActorMessages::Delete(msg), state.buckets.as_ref().unwrap()).await 
    }
}

pub async fn start_bidon() -> BidonActorChannel {
    let bidon = BidonActor{}.start();
    
    let (tx, rx) = oneshot::channel::<Return>();
    bidon.send(BidonActorMessages::CreateNS(MsgParams{
        ns: "default".to_string(),
        key: None,
        value: None,
        ch: tx,
    })).await.unwrap();
    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NoError => {},
                _ => { panic!() },
            };
        },
        Err(_e) => { panic!() }
    };

    let (tx, rx) = oneshot::channel::<Return>();
    bidon.send(BidonActorMessages::LoadNS(MsgParams{
        ns: "default".to_string(),
        key: None,
        value: None,
        ch: tx,
    })).await.unwrap();
    match rx.await {
        Ok(r) => {
            match r.err {
                Errors::NoError => {},
                Errors::NotFound => {},
                _ => { panic!() },
            };
        },
        Err(_e) => { panic!() }
    };


    bidon
}

