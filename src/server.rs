use std::collections::HashMap;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

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
        // To make our logic simple and consistency, we will broadcast to all
        // users which include msg sender.
        // On frontend, sender will send msg and receive its broadcasted msg
        // and then show his msg on frontend page.
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


#[derive(Default)]
pub struct MyGreeter {
    shared: Arc<RwLock<Shared>>,
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

        // let (tx, rx) = mpsc::channel(4);
        // let greets = self.greets.clone();
    
        // tokio::spawn(async move {
        //     for greet in &greets[..] {
        //         println!("Sending greet: {:?}", greet);
        //         tx.send(Ok(greet.clone())).await.unwrap();
        //     }
        // });
    
        // Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ExtractEntitiesStream = ReceiverStream<Result<ExtractEntitiesResponse, Status>>;

    async fn extract_entities(
        &self,
        _: Request<tonic::Streaming<ExtractEntitiesRequest>>,
    ) -> Result<Response<Self::ExtractEntitiesStream>, Status> {
        Err(Status::unimplemented("Not implemented yet"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50052".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreetServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}