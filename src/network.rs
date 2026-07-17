use std::{
    collections::HashSet,
    io::{self, BufRead, Write},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use bevy::prelude::*;
use tokio::{
    runtime::Builder,
    sync::mpsc as tokio_mpsc,
    time::{self, MissedTickBehavior},
};
use util::Conn;
use webrtc_ice::{
    agent::{Agent, agent_config::AgentConfig},
    candidate::{Candidate, candidate_base::unmarshal_candidate},
    mdns::MulticastDnsMode,
    network_type::NetworkType,
    url::Url,
};

const DEFAULT_STUN_SERVER: &str = "stun.l.google.com:19302";
const PACKET_MAGIC: &[u8; 4] = b"C4P1";
const TOKEN_SIZE: usize = 16;

#[derive(Clone, Default, Resource, Debug)]
pub enum NetworkMode {
    #[default]
    Offline,
    Host,
    Join(String),
}

impl NetworkMode {
    pub fn from_args() -> Self {
        let mut args = std::env::args().skip(1);
        match args.next().as_deref() {
            Some("--host") => Self::Host,
            Some("--join") => match args.next() {
                Some(offer) => Self::Join(offer),
                None => {
                    eprintln!("Paste the host offer code, then press Enter:");
                    let mut offer = String::new();
                    if io::stdin().lock().read_line(&mut offer).is_ok() {
                        Self::Join(offer.trim().to_owned())
                    } else {
                        Self::Offline
                    }
                }
            },
            Some(other) => {
                eprintln!("Unknown argument `{other}`. Starting offline.");
                Self::Offline
            }
            None => Self::Offline,
        }
    }

    fn role(&self) -> Option<NetworkRole> {
        match self {
            Self::Offline => None,
            Self::Host => Some(NetworkRole::Host),
            Self::Join(_) => Some(NetworkRole::Guest),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkRole {
    Host,
    Guest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Offline,
    WaitingForPeer,
    Punching,
    Connected,
    Failed,
}

#[derive(Resource)]
pub struct NetworkStatus {
    pub role: Option<NetworkRole>,
    pub connected: bool,
    pub state: ConnectionState,
}

impl NetworkStatus {
    fn for_mode(mode: &NetworkMode) -> Self {
        Self {
            role: mode.role(),
            connected: false,
            state: match mode {
                NetworkMode::Offline => ConnectionState::Offline,
                NetworkMode::Host => ConnectionState::WaitingForPeer,
                NetworkMode::Join(_) => ConnectionState::Punching,
            },
        }
    }
}

pub enum NetworkCommand {
    Move { x: u8, y: u8 },
}

pub enum NetworkEvent {
    Connected,
    Move { x: u8, y: u8 },
    Error(String),
}

#[derive(Resource, Default)]
pub struct NetworkInbox {
    pub events: Vec<NetworkEvent>,
}

#[derive(Resource)]
pub struct NetworkHandle {
    pub commands: mpsc::Sender<NetworkCommand>,
    events: Mutex<mpsc::Receiver<NetworkEvent>>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkReceiveSet;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let mode = app.world().resource::<NetworkMode>().clone();
        let status = NetworkStatus::for_mode(&mode);
        let (command_sender, command_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();

        if !matches!(mode, NetworkMode::Offline) {
            thread::Builder::new()
                .name("connect4-network".to_owned())
                .spawn(move || {
                    if let Err(error) = run_network(mode, command_receiver, &event_sender) {
                        let _ = event_sender.send(NetworkEvent::Error(error.to_string()));
                    }
                })
                .expect("failed to start network thread");
        }

        app.insert_resource(NetworkHandle {
            commands: command_sender,
            events: Mutex::new(event_receiver),
        })
        .insert_resource(status)
        .init_resource::<NetworkInbox>()
        .configure_sets(Update, NetworkReceiveSet)
        .add_systems(Update, receive_network_events.in_set(NetworkReceiveSet));
    }
}

fn receive_network_events(
    handle: Res<NetworkHandle>,
    mut status: ResMut<NetworkStatus>,
    mut inbox: ResMut<NetworkInbox>,
) {
    let receiver = handle.events.lock().expect("network event lock poisoned");

    while let Ok(event) = receiver.try_recv() {
        match &event {
            NetworkEvent::Connected => {
                status.connected = true;
                status.state = ConnectionState::Connected;
                info!("Connected directly to the other player");
            }
            NetworkEvent::Error(message) => {
                status.connected = false;
                status.state = ConnectionState::Failed;
                error!("Network error: {message}");
            }
            NetworkEvent::Move { .. } => {}
        }
        inbox.events.push(event);
    }
}

#[derive(Clone, Debug)]
struct Description {
    token: [u8; TOKEN_SIZE],
    ufrag: String,
    pwd: String,
    candidates: Vec<String>,
}

fn encode_description(prefix: &str, description: &Description) -> String {
    let candidate_text = description.candidates.join("\n");
    format!(
        "{prefix}|{}|{}|{}|{}",
        encode_token(description.token),
        encode_text(&description.ufrag),
        encode_text(&description.pwd),
        encode_text(&candidate_text),
    )
}

fn decode_description(value: &str, expected_prefix: &str) -> io::Result<Description> {
    let fields: Vec<_> = value.trim().split('|').collect();
    if fields.len() != 5 || fields[0] != expected_prefix {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid Connect 4 connection code",
        ));
    }

    let candidate_text = decode_text(fields[4])?;
    let candidates = candidate_text
        .lines()
        .filter(|candidate| !candidate.is_empty())
        .map(str::to_owned)
        .collect();

    Ok(Description {
        token: decode_token(fields[1])?,
        ufrag: decode_text(fields[2])?,
        pwd: decode_text(fields[3])?,
        candidates,
    })
}

fn encode_text(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_text(value: &str) -> io::Result<String> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "connection code contains invalid encoded text",
        )
    })?;
    String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "connection code contains invalid UTF-8",
        )
    })
}

fn encode_token(token: [u8; TOKEN_SIZE]) -> String {
    token.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_token(value: &str) -> io::Result<[u8; TOKEN_SIZE]> {
    if value.len() != TOKEN_SIZE * 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "connection code contains an invalid token",
        ));
    }

    let mut token = [0; TOKEN_SIZE];
    for (index, byte) in token.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "connection code contains an invalid token",
            )
        })?;
    }
    Ok(token)
}

fn run_network(
    mode: NetworkMode,
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| io::Error::other(format!("could not start network runtime: {error}")))?;
    runtime.block_on(run_network_async(mode, commands, events))
}

async fn run_network_async(
    mode: NetworkMode,
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    match mode {
        NetworkMode::Offline => Ok(()),
        NetworkMode::Host => run_host(commands, events).await,
        NetworkMode::Join(offer) => run_guest(offer, commands, events).await,
    }
}

async fn run_host(
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    let local = gather_ice(NetworkRole::Host).await?;
    let offer = Description {
        token: create_token(),
        ufrag: local.ufrag.clone(),
        pwd: local.pwd.clone(),
        candidates: local.candidates.clone(),
    };

    println!("\nConnect 4 host offer (send this through chat or encode it as a QR code):");
    println!("{}", encode_description("C4O2", &offer));
    println!("Paste the guest answer code here and press Enter:");
    io::stdout().flush()?;

    let answer = read_stdin_line().await?;
    let answer = decode_description(&answer, "C4A2")?;
    if answer.token != offer.token {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "guest answer belongs to a different host offer",
        ));
    }

    run_session(local.agent, NetworkRole::Host, answer, commands, events).await
}

async fn run_guest(
    offer: String,
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    let host = decode_description(&offer, "C4O2")?;
    let local = gather_ice(NetworkRole::Guest).await?;
    let answer = Description {
        token: host.token,
        ufrag: local.ufrag.clone(),
        pwd: local.pwd.clone(),
        candidates: local.candidates.clone(),
    };

    println!("\nConnect 4 guest answer (send this back to the host):");
    println!("{}", encode_description("C4A2", &answer));
    io::stdout().flush()?;

    run_session(local.agent, NetworkRole::Guest, host, commands, events).await
}

struct LocalIce {
    agent: Arc<Agent>,
    ufrag: String,
    pwd: String,
    candidates: Vec<String>,
}

async fn gather_ice(role: NetworkRole) -> io::Result<LocalIce> {
    let stun_server =
        std::env::var("CONNECT4_STUN_SERVER").unwrap_or_else(|_| DEFAULT_STUN_SERVER.to_owned());
    let stun_url = Url::parse_url(&format!("stun:{stun_server}"))
        .map_err(|error| io::Error::other(format!("invalid STUN server: {error}")))?;

    gather_agent(role, vec![stun_url]).await
}

async fn gather_agent(role: NetworkRole, urls: Vec<Url>) -> io::Result<LocalIce> {
    let agent = Arc::new(
        Agent::new(AgentConfig {
            urls,
            network_types: vec![NetworkType::Udp4],
            multicast_dns_mode: MulticastDnsMode::Disabled,
            include_loopback: true,
            is_controlling: role == NetworkRole::Host,
            ..Default::default()
        })
        .await
        .map_err(ice_error)?,
    );

    let (candidate_sender, mut candidate_receiver) = tokio_mpsc::unbounded_channel();
    agent.on_candidate(Box::new(move |candidate| {
        let candidate_sender = candidate_sender.clone();
        Box::pin(async move {
            let _ = candidate_sender.send(candidate.map(|candidate| candidate.marshal()));
        })
    }));
    agent.gather_candidates().map_err(ice_error)?;

    let mut candidates = Vec::new();
    while let Some(candidate) = candidate_receiver.recv().await {
        let Some(candidate) = candidate else {
            break;
        };
        candidates.push(candidate);
    }
    if candidates.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "ICE did not find any local UDP candidates",
        ));
    }

    let (ufrag, pwd) = agent.get_local_user_credentials().await;
    Ok(LocalIce {
        agent,
        ufrag,
        pwd,
        candidates,
    })
}

async fn read_stdin_line() -> io::Result<String> {
    tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        Ok::<_, io::Error>(line)
    })
    .await
    .map_err(|error| io::Error::other(format!("could not read connection code: {error}")))?
}

fn ice_error(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("ICE error: {error}"))
}

#[derive(Clone, Copy)]
enum Packet {
    Move { sequence: u32, x: u8, y: u8 },
    Ack { sequence: u32 },
}

fn encode_packet(packet: Packet, token: [u8; TOKEN_SIZE]) -> Vec<u8> {
    let (kind, payload) = match packet {
        Packet::Move { sequence, x, y } => {
            let mut payload = sequence.to_be_bytes().to_vec();
            payload.extend([x, y]);
            (0, payload)
        }
        Packet::Ack { sequence } => (1, sequence.to_be_bytes().to_vec()),
    };

    let mut bytes = Vec::with_capacity(5 + TOKEN_SIZE + payload.len());
    bytes.extend_from_slice(PACKET_MAGIC);
    bytes.push(kind);
    bytes.extend_from_slice(&token);
    bytes.extend(payload);
    bytes
}

fn decode_packet(bytes: &[u8], token: [u8; TOKEN_SIZE]) -> Option<Packet> {
    if bytes.len() < 5 + TOKEN_SIZE || &bytes[..4] != PACKET_MAGIC {
        return None;
    }
    if bytes[5..5 + TOKEN_SIZE] != token {
        return None;
    }

    let payload = &bytes[5 + TOKEN_SIZE..];
    match bytes[4] {
        0 if payload.len() == 6 => Some(Packet::Move {
            sequence: u32::from_be_bytes(payload[..4].try_into().ok()?),
            x: payload[4],
            y: payload[5],
        }),
        1 if payload.len() == 4 => Some(Packet::Ack {
            sequence: u32::from_be_bytes(payload.try_into().ok()?),
        }),
        _ => None,
    }
}

struct PendingMove {
    sequence: u32,
    packet: Vec<u8>,
    sent_at: Instant,
    created_at: Instant,
}

async fn run_session(
    agent: Arc<Agent>,
    role: NetworkRole,
    remote: Description,
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    if remote.candidates.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "the connection code did not contain any ICE candidates",
        ));
    }

    for candidate in &remote.candidates {
        let candidate = unmarshal_candidate(candidate).map_err(ice_error)?;
        let candidate: Arc<dyn Candidate + Send + Sync> = Arc::new(candidate);
        agent.add_remote_candidate(&candidate).map_err(ice_error)?;
    }
    let remote_ufrag = remote.ufrag.clone();
    let remote_pwd = remote.pwd.clone();
    let token = remote.token;
    agent
        .set_remote_credentials(remote.ufrag, remote.pwd)
        .await
        .map_err(ice_error)?;

    let (_cancel_sender, cancel_receiver) = tokio_mpsc::channel(1);
    let conn: Arc<dyn Conn + Send + Sync> = match role {
        NetworkRole::Host => agent
            .dial(cancel_receiver, remote_ufrag, remote_pwd)
            .await
            .map_err(ice_error)?,
        NetworkRole::Guest => agent
            .accept(cancel_receiver, remote_ufrag, remote_pwd)
            .await
            .map_err(ice_error)?,
    };

    let _ = events.send(NetworkEvent::Connected);
    run_data_channel(conn, token, commands, events).await
}

async fn run_data_channel(
    conn: Arc<dyn Conn + Send + Sync>,
    token: [u8; TOKEN_SIZE],
    commands: mpsc::Receiver<NetworkCommand>,
    events: &mpsc::Sender<NetworkEvent>,
) -> io::Result<()> {
    let mut sequence = 0;
    let mut pending = Vec::new();
    let mut received_sequences = HashSet::new();
    let mut buffer = [0; 1024];
    let mut receive_tick = time::interval(Duration::from_millis(10));
    receive_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        while let Ok(command) = commands.try_recv() {
            match command {
                NetworkCommand::Move { x, y } => {
                    sequence += 1;
                    let packet = encode_packet(Packet::Move { sequence, x, y }, token);
                    conn.send(&packet).await.map_err(ice_error)?;
                    pending.push(PendingMove {
                        sequence,
                        packet,
                        sent_at: Instant::now(),
                        created_at: Instant::now(),
                    });
                }
            }
        }

        let now = Instant::now();
        for pending_move in &mut pending {
            if now.duration_since(pending_move.sent_at) >= Duration::from_millis(250) {
                let _ = conn.send(&pending_move.packet).await;
                pending_move.sent_at = now;
            }
        }
        pending.retain(|pending_move| {
            now.duration_since(pending_move.created_at) < Duration::from_secs(20)
        });

        tokio::select! {
            _ = receive_tick.tick() => {}
            result = conn.recv(&mut buffer) => {
                let size = result.map_err(ice_error)?;
                let Some(packet) = decode_packet(&buffer[..size], token) else {
                    continue;
                };
                match packet {
                    Packet::Move { sequence, x, y } => {
                        let acknowledgement = encode_packet(Packet::Ack { sequence }, token);
                        let _ = conn.send(&acknowledgement).await;
                        if received_sequences.insert(sequence) {
                            let _ = events.send(NetworkEvent::Move { x, y });
                        }
                    }
                    Packet::Ack { sequence } => {
                        pending.retain(|pending_move| pending_move.sequence != sequence);
                    }
                }
            }
        }
    }
}

fn create_token() -> [u8; TOKEN_SIZE] {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let process = u128::from(std::process::id());
    (now ^ (process << 64)).to_be_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_code_round_trips() {
        let description = Description {
            token: [0xabu8; TOKEN_SIZE],
            ufrag: "local-user".to_owned(),
            pwd: "local-password".to_owned(),
            candidates: vec!["candidate one".to_owned(), "candidate two".to_owned()],
        };

        let encoded = encode_description("C4O2", &description);
        assert_eq!(
            decode_description(&encoded, "C4O2").unwrap().token,
            description.token
        );
        assert_eq!(
            decode_description(&encoded, "C4O2").unwrap().ufrag,
            description.ufrag
        );
        assert_eq!(
            decode_description(&encoded, "C4O2").unwrap().pwd,
            description.pwd
        );
        assert_eq!(
            decode_description(&encoded, "C4O2").unwrap().candidates,
            description.candidates
        );
    }

    #[test]
    fn packets_round_trip() {
        let token = [0x42u8; TOKEN_SIZE];
        let packets = [
            Packet::Move {
                sequence: 7,
                x: 2,
                y: 3,
            },
            Packet::Ack { sequence: 9 },
        ];

        for packet in packets {
            let encoded = encode_packet(packet, token);
            assert!(decode_packet(&encoded, token).is_some());
        }
    }
}
