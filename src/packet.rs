use std::{/* any::Any, */collections::HashMap};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Packet {
    pub data: Vec<u8>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Packets {
    pub packets: HashMap<String, Packet>,
}

/* Ideally this would be implemented, but i dont wanna deal with proc macros
    or dirty tricks
*/
// pub struct DeserializedPackets {
//     packets: HashMap<String, Box<dyn Any>>
// }

impl Packets {
    pub fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Option<T> {
        if let Some(packet) = self.packets.get(&key.to_owned()) {
            Some(Packet::deserialize::<T>(&packet.data))
        } else {
            None
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize::<Self>(&self)
            .unwrap()
    }

    pub fn deserialize(data: &[u8]) -> Self {
        bincode::deserialize::<Self>(&data)
            .unwrap()
    }
}

impl Packet {
    /// Alias: serialize()
    pub fn new<T: Serialize>(data: T) -> Self {
        Self {
            data: bincode::serialize::<T>(&data).unwrap(),
        }
    }

    pub fn deserialize<T: for<'a> Deserialize<'a>>(data: &[u8]) -> T {
        bincode::deserialize::<T>(data)
            .unwrap()
    }
}