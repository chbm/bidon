use std::collections::HashMap;

use murray::actor;
use tokio::sync::{oneshot, mpsc};

type Value =  Vec<u8>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BucketErrors {
    NoError,
    Failure,
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
                match store.insert(msg.key, v) {
                    Some(prev) => { r.val = Some(prev) },
                    None => { r.val = None }
                }
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
                Some(prev) => { r.val = Some(prev) },
                None => { r.err = BucketErrors::NotFound }
            }
        } else { r.err = BucketErrors::Failure; };

        msg.ch.send(r).unwrap();
    }

    fn init(&self, state: &mut BucketActorState) {
        state.store = Some(HashMap::new());
    }
}

pub fn start_bucket(name: String) -> mpsc::Sender<BucketActorMessages> {
    let actor = BucketActor{}.start(&name);

    actor
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::{start_bucket, BucketMsgParams, BucketErrors, Return, BucketActorMessages, Value};
    use tokio::sync::{oneshot, mpsc};
    use tokio_test::{
        assert_err, assert_ok, assert_pending, assert_ready, assert_ready_err, assert_ready_ok,
    };

    #[tokio::test]
    async fn test_bucket_crud() {
        let bucket = start_bucket("default".to_string());

        let fixture = HashMap::from([
            ("foo", "foo"),
            ("bar", "bar"),
            ("xpto", "xpto"),
            ("long", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ]);

        for k in fixture.keys() {
            let (tx, rx) = oneshot::channel::<Return>();
            let b = bucket.clone();
            match fixture.get(k) {
                Some(v) => {
                    let m = BucketActorMessages::Put(BucketMsgParams{
                        key: k.to_string(),
                        value: Some(Value::from(*v)),
                        ch: tx,
                    });
                    assert_ok!(b.send(m).await);

                    let ret = assert_ok!(rx.await);
                    assert_eq!(ret.err, BucketErrors::NoError);
                    assert_eq!(ret.val, None);
                },
                None => {},
            }
        }

        for k in fixture.keys() {
            let (tx, rx) = oneshot::channel::<Return>();
            let b = bucket.clone();
            let expected = if let Some(expected) = fixture.get(k) { Value::from(*expected) } else { todo!() };
            let m = BucketActorMessages::Get(BucketMsgParams{
                key: k.to_string(),
                value: None,
                ch: tx,
            });
            assert_ok!(b.send(m).await);

            let ret = assert_ok!(rx.await);
            assert_eq!(ret.err, BucketErrors::NoError); 
            assert_eq!(ret.val, Some(expected));
        }
        
        {
            let (tx, rx) = oneshot::channel::<Return>();
            let b = bucket.clone();
            let expected = if let Some(expected) = fixture.get(&"bar") { Value::from(*expected) } else { todo!() };
            let m = BucketActorMessages::Put(BucketMsgParams{
                key: "bar".to_string(),
                value: Some(Value::from("new")),
                ch: tx,
            });
            assert_ok!(b.send(m).await);

            let ret = assert_ok!(rx.await);
            assert_eq!(ret.err, BucketErrors::NoError);
            assert_eq!(ret.val, Some(expected));
        }

        {
            let (tx, rx) = oneshot::channel::<Return>();
            let b = bucket.clone();
            let expected = if let Some(expected) = fixture.get(&"foo") { Value::from(*expected) } else { todo!() };
            let m = BucketActorMessages::Delete(BucketMsgParams{
                key: "foo".to_string(),
                value: None,
                ch: tx,
            });
            assert_ok!(b.send(m).await);

            let ret = assert_ok!(rx.await);
            assert_eq!(ret.err, BucketErrors::NoError);
            assert_eq!(ret.val, Some(expected));
        }

        for k in vec!("foo", "not") {
            let (tx, rx) = oneshot::channel::<Return>();
            let b = bucket.clone();
            let m = BucketActorMessages::Get(BucketMsgParams{
                key: k.to_string(),
                value: None,
                ch: tx,
            });
            assert_ok!(b.send(m).await);
           
            let ret = assert_ok!(rx.await);
            assert_eq!(ret.err, BucketErrors::NotFound);
        }
    }




}
