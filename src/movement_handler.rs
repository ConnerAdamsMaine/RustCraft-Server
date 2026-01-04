use anyhow::Result;

use crate::protocol::PacketReader;

/// Player movement packet types
#[derive(Debug, Clone)]
pub struct PlayerPosition {
    pub x:      f64,
    pub y:      f64,
    pub z:      f64,
    pub ground: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerLook {
    pub yaw:    f32,
    pub pitch:  f32,
    pub ground: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerPositionAndLook {
    pub x:      f64,
    pub y:      f64,
    pub z:      f64,
    pub yaw:    f32,
    pub pitch:  f32,
    pub ground: bool,
}

/// Parse movement packets from client
pub fn parse_movement_packet(packet_id: i32, data: &[u8]) -> Result<Option<MovementPacket>> {
    match packet_id {
        0x04 => {
            // Player Position packet
            let mut reader = PacketReader::new(data);
            let x = reader.read_double()?;
            let y = reader.read_double()?;
            let z = reader.read_double()?;
            let ground = reader.read_bool()?;
            Ok(Some(MovementPacket::Position(PlayerPosition { x, y, z, ground })))
        }
        0x05 => {
            // Player Look packet
            let mut reader = PacketReader::new(data);
            let yaw = reader.read_float()?;
            let pitch = reader.read_float()?;
            let ground = reader.read_bool()?;
            Ok(Some(MovementPacket::Look(PlayerLook { yaw, pitch, ground })))
        }
        0x06 => {
            // Player Position and Look packet
            let mut reader = PacketReader::new(data);
            let x = reader.read_double()?;
            let y = reader.read_double()?;
            let z = reader.read_double()?;
            let yaw = reader.read_float()?;
            let pitch = reader.read_float()?;
            let ground = reader.read_bool()?;
            Ok(Some(MovementPacket::PositionAndLook(PlayerPositionAndLook {
                x,
                y,
                z,
                yaw,
                pitch,
                ground,
            })))
        }
        _ => {
            // Other packets we don't handle yet
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub enum MovementPacket {
    Position(PlayerPosition),
    Look(PlayerLook),
    PositionAndLook(PlayerPositionAndLook),
}

impl MovementPacket {
    pub fn new_position(x: f64, y: f64, z: f64, ground: bool) -> Self {
        MovementPacket::Position(PlayerPosition { x, y, z, ground })
    }

    pub fn new_look(yaw: f32, pitch: f32, ground: bool) -> Self {
        MovementPacket::Look(PlayerLook { yaw, pitch, ground })
    }

    pub fn new_position_and_look(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, ground: bool) -> Self {
        MovementPacket::PositionAndLook(PlayerPositionAndLook {
            x,
            y,
            z,
            yaw,
            pitch,
            ground,
        })
    }
}
