extern crate hyper;
extern crate tokio;

use hyper::{rt::Executor, server::conn::Http, service::service_fn, Body, Request, Response};
use std::{cell::RefCell, convert::Infallible, net::SocketAddr, rc::Rc, thread};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Builder,
    sync::mpsc::{self, Receiver, Sender},
    task::LocalSet,
};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 6060));
    let tcp_listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await.unwrap();

        thread::spawn(move || {
            handler(tcp_stream);
        });
    }
}

fn handler(tcp_stream: TcpStream) {
    let tokio_rt = Builder::new_current_thread().build().unwrap();

    let local = LocalSet::new();

    local.block_on(&tokio_rt, async {
        // Request and response channels.
        let (request_tx, mut request_rx) = mpsc::channel(1); // (Sender<Request<Body>>, Reciever<Request<Body>>)
        let (response_tx, response_rx) = mpsc::channel(1); // (Sender<Response<Body>>, Reciever<Response<Body>>)

        // `http_driver` runs in a separate task. It sends hyper request to the current task.
        local.spawn_local(http_driver(tcp_stream, request_tx.clone(), response_rx));

        // The current tasks recieves sent request and sends a hyper response back.
        if let Some(request) = request_rx.recv().await {
            println!("\n\n>>>> HANDLER: Recieved request = {:?}", request);

            // let b = body::to_bytes(request.into_body()).await.unwrap(); // ¡Reading the body makes it work!

            let response = Response::new(Body::from("Hello!"));

            println!("\n\n>>>> HANDLER: Sending our response = {:?}", response);

            response_tx.try_send(response).unwrap();

            println!("\n\n>>>> HANDLER: Response sent");
        }

        request_rx.recv().await.unwrap(); // A dummy wait to prevent the
    })
}

async fn http_driver(
    tcp_stream: TcpStream,
    request_tx: Sender<Request<Body>>,
    response_rx: Receiver<Response<Body>>,
) {
    let response_rx = Rc::new(RefCell::new(response_rx));

    let final_request_tx = request_tx.clone();

    Http::new()
        .with_executor(LocalExecutor)
        .serve_connection(
            tcp_stream,
            service_fn(move |request| {
                let request_tx = request_tx.clone();
                let response_rx = Rc::clone(&response_rx);

                async move {
                    let mut response_rx = response_rx.borrow_mut();

                    let mut response = Response::default();

                    // Send request.
                    if let Err(err) = request_tx.send(request).await {
                        println!("\n\n>>>> HTTP_DRIVER: Error sending response = {:?}", err);
                        return Ok(response);
                    }

                    println!("\n\n>>>> HTTP_DRIVER: Waiting to recieve response!");

                    // Wait for response.
                    if let Some(resp) = response_rx.recv().await {
                        response = resp;
                    }

                    println!(
                        "\n\n>>>> HTTP_DRIVER: Response is never recieved! This is not printed!"
                    );

                    Ok::<_, Infallible>(response)
                }
            }),
        )
        .await
        .unwrap();

    // This final send to signal the handler that the tokio runtime can end.
    final_request_tx.send(Request::default()).await.unwrap();
}

#[derive(Clone)]
struct LocalExecutor;

impl<F> Executor<F> for LocalExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}
