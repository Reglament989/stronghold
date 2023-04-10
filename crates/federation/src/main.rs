use std::{net::SocketAddr, pin::Pin, sync::Arc};

use petgraph::Graph;
use petgraph::{algo::astar, stable_graph::NodeIndex};

use tokio::sync::{broadcast::Sender, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    codegen::futures_core::Stream, metadata::MetadataValue, transport::Server, Request, Response,
    Status, Streaming,
};

use entity::proto::{
    federation_service_server::{FederationService as IFederationService, FederationServiceServer},
    packet::Packet as PacketType,
    AcknowledgePacket, Host, Hosts, Packet,
};

pub struct FederationService {
    graph: Arc<RwLock<Graph<Host, u32>>>,
    me: NodeIndex, // Our address
    tx: Sender<Packet>,
}

impl FederationService {
    async fn new(host: Host) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(128);
        let graph: Arc<RwLock<Graph<Host, u32>>> = Default::default();
        let me = graph.write().await.add_node(host);
        Self { me, graph, tx }
    }
}

type ForwardStream = Pin<Box<dyn Stream<Item = Result<Packet, Status>> + Send>>;

#[tonic::async_trait]
impl IFederationService for FederationService {
    type ForwardStream = ForwardStream;

    /// Acknowledge a new host and return all known hosts to the new host
    async fn acknowledge(&self, request: Request<Host>) -> Result<Response<Hosts>, Status> {
        let mut graph = self.graph.write().await;
        let new_node = graph.add_node(request.into_inner());
        graph.extend_with_edges(&[(self.me, new_node)]);
        let knowledged = graph
            .raw_nodes()
            .iter()
            .map(|node| node.weight.clone())
            .collect::<Vec<_>>();
        Ok(Response::new(Hosts { hosts: knowledged }))
    }

    /// Request a path to a host knowledged by this host
    /// Used by the client before actuall forwarding, and might be cached
    async fn request_path(&self, request: Request<Host>) -> Result<Response<Hosts>, Status> {
        let mut graph = self.graph.write().await;
        // TODO: Is this clone necessary? might be a bottleneck. but write access while calculating path might be a huger bottleneck
        let target = graph.add_node(request.into_inner());
        let path = astar(
            &*graph,
            self.me,
            |finish| finish == target,
            |e| *e.weight(),
            |_| 0,
        );
        if let Some((_, path)) = path {
            let hosts = path
                .iter()
                .map(|node| graph[*node].clone())
                .collect::<Vec<_>>();
            Ok(Response::new(Hosts { hosts }))
        } else {
            Err(Status::not_found("No path found"))
        }
    }

    /// Forward a packet to the next hop in path
    /// If the last hop is reached, ack packet will be returned
    async fn forward(
        &self,
        request: Request<Streaming<Packet>>,
    ) -> Result<tonic::Response<Self::ForwardStream>, tonic::Status> {
        // Per request stream
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        // Input stream from client
        let remote_addr = request.remote_addr().unwrap();
        let mut request_stream = request.into_inner();
        // Inner stream of broadcaster
        let mut inner_receiver = self.tx.subscribe();
        let inner_sender = self.tx.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    while let Some(packet) = request_stream.message().await.unwrap() {
                        // Just forward this packet to the next hop
                        inner_sender.send(packet).unwrap();
                    }
                } => {}
                _ = async {
                    while let Ok(Packet{packet: Some(packet)}) = inner_receiver.recv().await {
                        match packet {
                            PacketType::Forward(mut packet) => {
                                packet.hop += 1;
                                if let Some(host) = packet.path.get(usize::try_from(packet.hop).unwrap()) {
                                    if host.addr == remote_addr.to_string() {
                                        if packet.path.last() == Some(host) {
                                            let ack = Packet{packet: Some(PacketType::Acknowledge(AcknowledgePacket{
                                                success: true,
                                                forward: Some(packet.clone())
                                            }))};
                                            inner_sender.send(ack).unwrap();
                                        }
                                        // Deliver to the next hop or receiver
                                        let _ = tx.send(Ok(Packet{packet: Some(PacketType::Forward(packet))})).await;
                                    }
                                }
                            },
                            PacketType::Acknowledge(mut ack_packet) => {
                                // IDK why prost have option on required field, hate this spec so much
                                let mut packet = ack_packet.forward.clone().unwrap();
                                packet.hop -= 1;
                                if let Some(host) = packet.path.get(packet.hop as usize) {
                                    if host.addr == remote_addr.to_string() {
                                        ack_packet.forward = Some(packet);
                                        // Deliver to the -next hop or receiver
                                        let _ = tx.send(Ok(Packet{packet: Some(PacketType::Acknowledge(ack_packet))})).await;
                                    }
                                }
                            },
                        }
                    }
                } => {}
            }
            dbg!("client disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::ForwardStream))
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // let mut g: Graph<HostAddrDef, i32> = Graph::new();
    // let a = g.add_node(HostAddrDef("24.238.168.69:56914".parse()?));
    // let b = g.add_node(HostAddrDef("78.169.117.105:14882".parse()?));
    // g.extend_with_edges(&[(a, b)]);
    // let path = astar(&g, a, |finish| finish == e, |e| *e.weight(), |_| 0);
    // dbg!(path);
    // return Ok(());
    // dbg!(&packet);
    // let graph = *graph.read().await;
    // let target = graph.add_node(packet.target.unwrap());
    // let path = astar(&graph, me, |finish| finish == target, |e| *e.weight(), |_| 0);
    // if let Some((_, path)) = path {
    // let mut packet = packet;
    // packet.path = path.iter().map(|node| node.weight.clone()).collect();
    // let _ = inner_sender.send(packet).await;
    // }
    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(entity::proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let addr: SocketAddr = "127.0.0.1:50051".parse()?;
    let host = Host {
        addr: addr.to_string(),
        forwarder: true,
        last_seen: None,
    };
    let server = FederationService::new(host).await;

    let svc = FederationServiceServer::new(server);

    Server::builder()
        .add_service(reflector)
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}

/// Federation will be trust each others by signing the hash(packet) with list of allowed public keys hosts
/// Someone can be only forwarder they didnt receive packets from other hosts
/// Maybe for better security, each node can resign the packet, because receiver can check if the packet is trusted
fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
