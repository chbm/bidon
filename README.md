# BIDON

(ie. a rust bucket)

## What ? 

This is small KV-store with a REST/HTTP interface. It's based on [tokio](https://tokio.rs/) and [axum](https://docs.rs/axum/latest/axum/index.html). 
It's fully async and actor based using [murray](https://github.com/chbm/murray). You can find an intro about Actors on murray's readme.

## Why ? 

Because I felt like it. 

## How ?

`main` starts a "default" bucket and adds it to the API web app. The bucket is totally self contained in its tokio task and only exposes it's incoming messages channel. The channel is added as state on the web app so the routing handlers can send the bucket messages. This is unidirectional, so the messages include a `oneshot` channel the bucket uses to send back the response. Notice this is totally decoupled, whoever owns the rx side of the channel gets the response. 

The messages incoming to the bucket are totally ordered by the `mpsc::channel` and the actor is single threaded so responses are garanteed to be emited in the same order as the commands. However, there's no garantee about the ordering of consuption of the responses by the router handlers (or the networking between the server and the client) so from the outside (ie. http clients) we may see causality violations as the response to a previous command arrives after a later one. 

