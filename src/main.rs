use adapter::{Document, Store};
use log::Log;
use serde::{Deserialize, Serialize};

use crate::adapter::{DocsoreDocument, Filter, Relation};

pub mod adapter;
mod logger;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Person {
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pet {
    pet_name: String,
    voice: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Game {
    name: String,
    id: String,
}

impl Document for Game {
    fn collection() -> &'static str {
        "games"
    }

    fn indexes(&self) -> Vec<adapter::HashIndex> {
        vec![seahash::hash(self.name.as_bytes()).to_ne_bytes().to_vec()]
    }

    fn id(&self) -> Vec<u8> {
        (&self.id.as_bytes()).to_vec()
    }
}

impl Document for Pet {
    fn collection() -> &'static str {
        "pets"
    }

    fn indexes(&self) -> Vec<adapter::HashIndex> {
        vec![seahash::hash(self.pet_name.as_bytes())
            .to_ne_bytes()
            .to_vec()]
    }

    fn id(&self) -> Vec<u8> {
        (&self.pet_name.as_bytes()).to_vec()
    }
}

impl Document for Person {
    fn collection() -> &'static str {
        "test"
    }

    fn indexes(&self) -> Vec<adapter::HashIndex> {
        vec![seahash::hash(self.name.as_bytes()).to_ne_bytes().to_vec()]
    }

    fn id(&self) -> Vec<u8> {
        (&self.name.as_bytes()).to_vec()
    }
}

fn main() -> anyhow::Result<()> {
    logger::init()?;

    let store = Store::default();
    let all = std::time::Instant::now();
    let mut game = Game {
        name: "Another Game".into(),
        id: nanoid::nanoid!(),
    };
    for _ in 0..1000 {
        game.save(&store)?;

        game.index(&store)?;
        game.id = nanoid::nanoid!();
    }
    dbg!(all.elapsed());
    Ok(())
}
