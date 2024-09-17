use std::{net::SocketAddr, str::FromStr};

use chia::{
    protocol::{Handshake, Message, NewPeak, NewTransaction, NodeType, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{create_tls_connector, load_ssl_cert, ClientError, NetworkId, Peer};
use native_tls::TlsConnector;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cert = load_ssl_cert("wallet.crt", "wallet.key")?;
    let tls = create_tls_connector(&cert)?;

    // `dig dns-introducer.chia.net` for example to find peers
    let (peer, mut receiver) = connect_peer(
        NetworkId::Mainnet,
        tls,
        SocketAddr::from_str("188.74.105.141:8444")?,
    )
    .await?;

    println!("Connected to {}", peer.socket_addr());

    while let Some(message) = receiver.recv().await {
        if message.msg_type == ProtocolMessageTypes::NewPeak {
            let message = NewPeak::from_bytes(&message.data)?;
            println!("{:?}", message);
        } else if message.msg_type == ProtocolMessageTypes::NewTransaction {
            let message = NewTransaction::from_bytes(&message.data)?;
            println!("{:?}", message);
        }
    }

    Ok(())
}

pub async fn connect_peer(
    network_id: NetworkId,
    tls_connector: TlsConnector,
    socket_addr: SocketAddr,
) -> Result<(Peer, mpsc::Receiver<Message>), ClientError> {
    let (peer, mut receiver) = Peer::connect(socket_addr, tls_connector).await?;

    peer.send(Handshake {
        network_id: network_id.to_string(),
        protocol_version: "0.0.37".to_string(),
        software_version: "0.0.0".to_string(),
        server_port: 0,
        node_type: NodeType::FullNode,
        capabilities: vec![
            (1, "1".to_string()),
            (2, "1".to_string()),
            (3, "1".to_string()),
        ],
    })
    .await?;

    let Some(message) = receiver.recv().await else {
        return Err(ClientError::MissingHandshake);
    };

    if message.msg_type != ProtocolMessageTypes::Handshake {
        return Err(ClientError::InvalidResponse(
            vec![ProtocolMessageTypes::Handshake],
            message.msg_type,
        ));
    }

    let handshake = Handshake::from_bytes(&message.data)?;

    if handshake.node_type != NodeType::FullNode {
        return Err(ClientError::WrongNodeType(
            NodeType::FullNode,
            handshake.node_type,
        ));
    }

    if handshake.network_id != network_id.to_string() {
        return Err(ClientError::WrongNetwork(
            network_id.to_string(),
            handshake.network_id,
        ));
    }

    Ok((peer, receiver))
}
