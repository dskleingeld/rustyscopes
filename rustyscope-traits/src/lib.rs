#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<'a> {
    Data(&'a[u8])
}
