use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};
use bevy::app::{App, FixedUpdate, Update};
use bevy::prelude::{EventReader, ResMut};
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent};
use bevy::log::*;
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use bevy_renet::transport::{NetcodeClientPlugin, NetcodeServerPlugin};

fn send_server_message_system(mut server: ResMut<RenetServer>) {
    let channel_id = 0;
    server.broadcast_message(DefaultChannel::ReliableOrdered, "server message");
}

fn receive_server_message_system(mut server: ResMut<RenetServer>) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            info!("Received message {:?}", message);
        }
    }
}

fn handle_server_events_system(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected {client_id} => {
                info!("Client {client_id} connected");
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {client_id} disconnected: {reason}");
            }
        }
    }
}

fn send_client_message_system(mut client: ResMut<RenetClient>) {
    client.send_message(DefaultChannel::ReliableOrdered, "server message");
}

fn receive_client_message_system(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        info!("Received message {:?}", message);
    }
}

pub fn client_plugin(app: &mut App) {
    let client = RenetClient::new(ConnectionConfig::default());

    let authentication = ClientAuthentication::Unsecure {
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        client_id: 0,
        user_data: None,
        protocol_id: 0
    };
    let socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let mut transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();


    app.
        add_plugins(RenetClientPlugin)
        .add_plugins(NetcodeClientPlugin)
        .insert_resource(client)
        .insert_resource(transport)
        .add_systems(FixedUpdate, send_client_message_system)
        .add_systems(FixedUpdate, receive_client_message_system);
}

pub fn server_plugin(app: &mut App) {
    let server = RenetServer::new(ConnectionConfig::default());

    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    app.add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        .insert_resource(server)
        .insert_resource(transport)
        .add_systems(FixedUpdate, receive_server_message_system)
        .add_systems(FixedUpdate, handle_server_events_system)
        .add_systems(FixedUpdate, send_server_message_system);
}
