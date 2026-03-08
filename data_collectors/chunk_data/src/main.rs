use log::{debug, error, trace, LevelFilter};
use sandstone::network::CraftConnection;
use sandstone::protocol::game::player::ClientStatusAction;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::{AcknowledgeFinishConfigurationPacket, ClientCommandPacket, ConfirmTeleportPacket, HandshakingPacket, LoginAcknowledgedPacket, LoginStartPacket, Packet, ServerboundKeepAlivePacket, ServerboundKnownPacksPacket};
use sandstone::protocol::serialization::serializer_types::PrefixedArray;
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use tokio::net::TcpStream;
use uuid::Uuid;

/// This demonstrates the login sequence from a client perspective.
///
/// View the README for more information on how to run this example.
#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    debug!("Starting client");

    let socket = TcpStream::connect("127.0.0.1:25565").await.unwrap();

    // Create the client from the socket
    let mut client = CraftConnection::from_connection(socket, PacketDirection::CLIENT).unwrap();

    let handshake = Packet::Handshaking(HandshakingPacket {
        protocol_version: VarInt(ProtocolVerison::latest().get_version_number() as i32),
        server_address: "127.0.0.1".to_string(),
        port: 25565,
        next_state: VarInt(2),
    });

    debug!("Sending handshake packet: {handshake:?}");
    client.send_packet(handshake).await.unwrap();
    
    let login_start = Packet::LoginStart(LoginStartPacket {
        username: "dec4234".to_string(),
        uuid: Uuid::from_str("ef39c197-3c3d-4776-a226-22096378a966").unwrap(),
    });

    debug!("Sending login start packet: {login_start:?}");
    client.send_packet(login_start).await.unwrap();

    client.change_state(PacketState::LOGIN);

    let login_success = client.receive_packet().await.unwrap();

    match login_success {
        Packet::LoginSuccess(packet) => {
            debug!("Login successful: UUID: {}, Username: {}", packet.uuid, packet.username);
        }
        _ => {
            panic!("Unexpected packet received instead of login success: {login_success:?}");
        }
    }

    let login_ack = Packet::LoginAcknowledged(LoginAcknowledgedPacket {});
    client.send_packet(login_ack).await.unwrap();
    debug!("Sending login acknowledged packet");

    client.change_state(PacketState::CONFIGURATION);

    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::ClientboundPluginMessage(_) => {
                debug!("Received clientbound plugin message: {packet:?}");
                continue;
            }
            Packet::FeatureFlags(_) => {
                debug!("Received feature flags: {packet:?}");
                continue;
            }
            Packet::ClientboundKnownPacks(_) => {
                debug!("Received known packs: {packet:?}");
                break;
            }
            _ => {
                panic!("Received unexpected packet: {packet:?}");
            }
        }
    }

    let serverbound_known_packs = Packet::ServerboundKnownPacks(ServerboundKnownPacksPacket {
        entries: PrefixedArray::new(vec![]),
    });

    debug!("Sending serverbound known packs: {serverbound_known_packs:?}");
    client.send_packet(serverbound_known_packs).await.unwrap();

    loop {
        let packet = client.receive_packet().await;

        if let Ok(packet) = packet {
            match packet {
                Packet::RegistryData(pack) => {
                    trace!("Received registry data: {pack:?}");

                    continue;
                }
                Packet::UpdateTags(_) => {
                    trace!("Received update tags");
                    continue;
                }
                Packet::FinishConfiguration(fc) => {
                    debug!("Received finish configuration packet: {fc:?}");
                    break;
                }
                _ => {
                    panic!("Received unexpected packet: {packet:?}");
                }
            }
        } else {
            error!("Failed to receive packet: {packet:?}");
        }
    }

    let ack_config = Packet::AcknowledgeFinishConfiguration(AcknowledgeFinishConfigurationPacket {});
    debug!("Sending acknowledge finish configuration packet: {ack_config:?}");
    client.send_packet(ack_config).await.unwrap();

    client.change_state(PacketState::PLAY);

    let packet = client.receive_packet().await.unwrap();
    match packet {
        Packet::LoginInfo(l) => {
            debug!("Received login info: {l:?}");
        }
        _ => {
            panic!("Expected acknowledge finish configuration, got: {packet:?}");
        }
    }

    // optional packets
    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::ChangeDifficulty(cd) => {
                debug!("Received change difficulty: {cd:?}");
                continue;
            }
            Packet::PlayerAbilities(pa) => {
                debug!("Received player abilities: {pa:?}");
                continue;
            }
            Packet::SetHeldItem(shi) => {
                debug!("Received set held item: {shi:?}");
                continue;
            }
            Packet::UpdateRecipes(_) => {
                debug!("Received update recipes.");
                continue;
            }
            Packet::EntityEvent(ee) => {
                debug!("Received entity event: {ee:?}");
                continue;
            }
            Packet::CommandsGraph(cg) => {
                debug!("Received commands graph: {cg:?}");
                continue;
            }
            Packet::RecipeBookSettings(rb) => {
                debug!("Received recipe book settings: {rb:?}");
                continue;
            }
            Packet::RecipeBookAdd(rad) => {
                debug!("Received recipe book add: {rad:?}");
                break;
            }
            _ => {
                panic!("Received unexpected packet: {packet:?}");
            }
        }
    }

    let teleport_id;

    match client.receive_packet().await.unwrap() {
        Packet::SyncPlayerPosition(spp) => {
            teleport_id = spp.teleport_id;
            debug!("Received known packs: {spp:?}");
        }
        packet => {
            panic!("Received unexpected packet: {packet:?}");
        }
    }

    let confirm = Packet::ConfirmTeleport(ConfirmTeleportPacket::new(teleport_id));
    debug!("Sending confirm teleport packet: {confirm:?}");
    client.send_packet(confirm).await.unwrap();

    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::ServerData(sd) => { // todo: fix
                debug!("Received server data: {:?}", sd.motd);
                continue;
            }
            Packet::PlayerInfoUpdate(piu) => {
                debug!("Received player info update: {piu:?}");
                continue;
            }
            Packet::InitializeWorldBorder(iwb) => {
                debug!("Received initialize world border: {iwb:?}");
                continue;
            }
            Packet::UpdateTime(ut) => {
                debug!("Received update time: {ut:?}");
                continue;
            }
            Packet::SetDefaultSpawnPosition(sds) => {
                debug!("Received set default spawn position: {sds:?}");
                continue;
            }
            Packet::GameEvent(ge) => { // required unlike the rest
                debug!("Received game event: {ge:?}");
                continue;
            }
            Packet::SetTickingState(st) => {
                debug!("Received set ticking state: {st:?}");
                continue;
            }
            Packet::StepTick(st) => {
                debug!("Received step tick: {st:?}");
                continue;
            }
            Packet::SetCenterChunk(scc) => {
                debug!("Received set center chunk: {scc:?}");
                continue;
            }
            Packet::SetContainerContent(scc) => {
                debug!("Received set container content: {scc:?}");
                continue;
            }
            Packet::SetEntityMetadata(sem) => {
                debug!("Received set entity metadata: {sem:?}");
                continue;
            }
            Packet::UpdateAttributes(ua) => {
                debug!("Received update attributes: {ua:?}");
                continue;
            }
            Packet::UpdateAdvancements(_) => {
                debug!("Received update advancements.");
                continue;
            }
            Packet::EntityEvent(ee) => {
                debug!("Received entity event: {ee:?}");
                continue;
            }
            Packet::SetHealth(sh) => {
                debug!("Received set health: {sh:?}");

                if sh.health <= 0.0 {
                    let respawn = Packet::ClientCommand(ClientCommandPacket {
                        action: ClientStatusAction::PerformRespawn,
                    });

                    debug!("Sending client command (respawn).");
                    client.send_packet(respawn).await.unwrap();
                }

                continue;
            }
            Packet::SetExperience(se) => {
                debug!("Received set experience: {se:?}");
                continue;
            }
            Packet::ClientboundKeepAlive(ka) => {
                debug!("Received clientbound keep alive: {ka:?}");

                let keep_alive = Packet::ServerboundKeepAlive(ServerboundKeepAlivePacket {
                    keep_alive_id: ka.keep_alive_id,
                });

                debug!("Sending serverbound keep alive: {keep_alive:?}");
                client.send_packet(keep_alive).await.unwrap();

                continue;
            }
            Packet::PlayerAbilities(pa) => {
                debug!("Received player abilities: {pa:?}");
                continue;
            }
            Packet::ChunkBatchStart(_) => {
                debug!("Received chunk batch start.");
                break;
            }
            Packet::DisconnectPlay(dp) => {
                debug!("Disconnected: {dp:?}");
                return;
            }
            _ => {
                panic!("Received unexpected packet: {packet:?}");
            }
        }
    }

    std::fs::create_dir_all("chunk_output").unwrap();

    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::ChunkDataUpdateLight(cbd) => {
                let filename = format!("chunk_output/chunk_{}_{}.txt", cbd.x, cbd.z);
                debug!("Writing chunk data to {filename}");
                std::fs::write(&filename, format!("{cbd:#?}")).unwrap();
                continue;
            }
            Packet::ChunkBatchFinished(cbf) => {
                debug!("Finished receiving {} chunks.", cbf.size.0);
                return;
            }
            Packet::DisconnectPlay(dp) => {
                debug!("Disconnected: {dp:?}");
                return;
            }
            _ => {
                panic!("Received unexpected packet: {packet:?}");
            }
        }
    }
}
