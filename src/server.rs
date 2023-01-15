use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use tokio::task::spawn_blocking;
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
use tokenizers::tokenizer::{Tokenizer};
use async_stream::stream;

use greetv1::extract_entities_response;
use greetv1::greet_service_server::{GreetService, GreetServiceServer};
use greetv1::{GreetResponse, GreetRequest, GreetStreamResponse, GreetStreamRequest, ExtractEntitiesRequest, ExtractEntitiesResponse};

pub mod greetv1 {
    tonic::include_proto!("greet.v1");
}

// When a new user connects, we will create a pair of mpsc channel.
// Add the users and its related senders will be saved in below shared struct
#[derive(Debug)]
#[derive(Default)]
struct Shared {
    senders: HashMap<Uuid, mpsc::Sender<GreetStreamResponse>>,
}

impl Shared {
    async fn broadcast(&self, msg: GreetStreamResponse) {
        for (name, tx) in &self.senders {
            match tx.send(msg.clone()).await {
                Ok(_) => {}
                Err(_) => {
                    println!("[Broadcast] SendError: to {}, {:?}", name, msg)
                }
            }
        }
    }
}

pub struct MyGreeter {
    shared: Arc<RwLock<Shared>>,
    tokenizer: Tokenizer,
}

impl MyGreeter {
    fn new (tokenizer: Tokenizer) -> Self {
        let tokenizer = tokenizer;
        Self {
            shared: Arc::new(RwLock::new(Shared::default())),
            tokenizer: tokenizer,
        }
    }
}

#[tonic::async_trait]
impl GreetService for MyGreeter {
    async fn greet(
        &self,
        request: Request<GreetRequest>,
    ) -> Result<Response<GreetResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let name = request.into_inner().name;
        
        self.shared.read().await.broadcast(GreetStreamResponse { people: (name.clone()) }).await;

        let reply = greetv1::GreetResponse {
            greeting: format!("Hello {}!", name),
        };
        Ok(Response::new(reply))
    }

    type GreetStreamStream = ReceiverStream<Result<GreetStreamResponse, Status>>;

    async fn greet_stream(
        &self,
        request: Request<GreetStreamRequest>,
    ) -> Result<Response<Self::GreetStreamStream>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let (stream_tx, stream_rx) = mpsc::channel(1); // Fn usage

        // When connecting, create related sender and reciever
        let (tx, mut rx) = mpsc::channel(1);
        let name = Uuid::new_v4();
        {
            self.shared.write().await.senders.insert(name, tx);
        }

        let shared_clone = self.shared.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match stream_tx.send(Ok(msg)).await {
                    Ok(_) => {}
                    Err(_) => {
                        // If sending failed, then remove the user from shared data
                        println!(
                            "[Remote] stream tx sending error."
                        );
                        shared_clone.write().await.senders.remove(&name);
                    }
                }
            }
        });

        Ok(Response::new(
            tokio_stream::wrappers::ReceiverStream::new(stream_rx),
        ))
    }

    // type ExtractEntitiesStream = Pin<Box<dyn Stream<Item = Result<ExtractEntitiesResponse, Status>> + Send + Sync + 'static>>;
    type ExtractEntitiesStream = Pin<Box<dyn Stream<Item = Result<ExtractEntitiesResponse, Status>> + Send + 'static>>;

    async fn extract_entities(
        &self,
        request: Request<tonic::Streaming<ExtractEntitiesRequest>>,
    ) -> Result<Response<Self::ExtractEntitiesStream>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let mut stream = request.into_inner();

        let shared_tokenizer = self.tokenizer.clone();
        
        let output = stream! {
            loop {
                let msg = stream.message().await;
                let msg = match msg {
                    Ok(msg) => msg.unwrap().message,
                    Err(_) => break,
                };

                let enc = shared_tokenizer.encode(msg, false).unwrap();
                println!("{:?}", enc.get_tokens());

                yield Ok(ExtractEntitiesResponse {
                    results: vec!{
                        extract_entities_response::Result {
                            text: "hello".to_string(),
                            label: "world".to_string(),
                        }
                    },
                });
            }
        };

        Ok(Response::new(Box::pin(output) as Self::ExtractEntitiesStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50052".parse().unwrap();
    let tokenizer = spawn_blocking(move || Tokenizer::from_pretrained("bert-base-cased", None).unwrap()).await.unwrap();
    let greeter = MyGreeter::new(tokenizer);

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreetServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}