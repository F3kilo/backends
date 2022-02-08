use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::{Arc, Mutex};

use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Default, Clone)]
pub struct CountersTransform {

}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for CountersTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CountersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CountersMiddleware {
            counters: self.counters.clone(),
            service,
        }))
    }
}

pub struct CountersMiddleware<S> {
    counters: Arc<Counters>,
    service: S,
}

impl<S, B> Service<ServiceRequest> for CountersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let addr = req.peer_addr().map(|a| a.to_string()).unwrap_or_else(|| "unknown".to_string());
        self.counters.increase(&addr);
        let count = self.counters.get(&addr);
        log::info!("It's your {count} request");

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

#[derive(Default)]
struct Counters(Mutex<HashMap<String, u64>>);

impl Counters {
    pub fn increase(&self, key: &str) {
        let mut map = self.0.lock().unwrap();
        match map.entry(key.to_string()) {
            Entry::Occupied(mut v) => *v.get_mut() += 1,
            Entry::Vacant(v) => {
                v.insert(1);
            }
        }
    }

    pub fn get(&self, key: &str) -> u64 {
        let map = self.0.lock().unwrap();
        map.get(key).copied().unwrap_or(0)
    }
}
