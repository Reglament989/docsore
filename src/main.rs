use adapter::Store;
use serde::{Deserialize, Serialize};

use crate::adapter::DocsoreDocument;

pub mod adapter;
mod logger;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Test {
    name: String,
}

fn main() -> anyhow::Result<()> {
    logger::init()?;

    let store = Store::default();

    let data = Test {
        name: "test".into(),
    };

    let mut document: DocsoreDocument<Test> = data.clone().into();

    let key = store.collection("test")?.put(&mut document)?;

    dbg!(store.collection("test")?.get::<Test>(&key)?);

    let mut document: DocsoreDocument<Test> = data.into();

    document.make_relation("friends", key);

    let key = store.collection("test")?.put(&mut document)?;

    let document = store.collection("test")?.get::<Test>(&key)?;

    dbg!(document.relation::<&str, Test>("friends", &store.collection("test")?)?);

    dbg!(store.collection("test")?.count()?);
    dbg!(seahash::hash(b"test"));
    Ok(())
}
