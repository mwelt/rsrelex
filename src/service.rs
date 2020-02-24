use super::types::{Env, DipreInput, AsyncLogger};
use super::dipre::do_dipre;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::http::{Request, Response, Error};
use hyper::{Server, Body};
use hyper::service::{make_service_fn, service_fn};
use async_trait::async_trait;

//use futures::executor::ThreadPool;

struct ALogger {
    sender: hyper::body::Sender,
    conn_valid: bool
}

#[async_trait]
impl AsyncLogger for ALogger {
    async fn log(&mut self, mut s: String) {
        println!("{}", s);
        s.push_str("\n");
        //TODO: check for error and mark the 
        //connection failure, to ignore this on further calls
        self.sender.send_data(s.into()).await;
    }
}

async fn handle_client(_req: Request<Body>, env: Arc<Env>) 
    -> Result<Response<Body>, Error> {

    let body = hyper::body::to_bytes(_req).await;
    if let Ok(body) = body {
        let di: Result<DipreInput, _> = serde_json::from_slice(&body);
        if let Ok(di) = di {
            let (sender, body) = Body::channel();

            let logger = ALogger {
                sender
            };

            let env = env.clone();

            let calc = async move {
                //sender.send_data("Hello World".to_string().into()).await;
                //logger.log("Hello World".to_string()).await;
                do_dipre(di, env.as_ref(), logger).await;
            };

            tokio::spawn(calc);

            Response::builder()
                .status(hyper::StatusCode::OK).body(body)
        } else {
            Response::builder()
                .status(400)
                .body("Unable to parse JSON data.".into())
        }
    } else {
        Response::builder()
            .status(400)
            .body("Unable to read body data.".into())
    }
}

// struct ServerEnv {
//     pool: ThreadPool
// }

// impl ServerEnv {
//     fn new() -> ServerEnv {
//         let pool = ThreadPool::builder().pool_size(2).create()
//             .expect("unable to create thread pool");
//         ServerEnv {
//             pool
//         }
//     }
// }

//SERVER
pub async fn run_server(env: Env){
    
    let env = Arc::new(env);
    // let senv = Arc::new(ServerEnv::new()); 

    // save a reference for later use
    // let senv_ = senv.clone();

    let addr = SocketAddr::from(([127,0,0,1], 2323));

    let make_svc = make_service_fn(move |_| {
        // let senv = senv.clone();
        let env = env.clone(); 
        async move {
            Ok::<_, Infallible>(service_fn(move |_req| {
                let env = env.clone();
                // let senv = senv.clone();
                //https://github.com/hyperium/hyper/blob/master/examples/state.rs
                async move {
                    // let senv = senv.as_ref();
                    handle_client(_req, env).await
                }
            }))}
    });

    let server = Server::bind(&addr).serve(make_svc);

    // let f = async {
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    // };

    //senv_.as_ref().pool.run(f);
}
