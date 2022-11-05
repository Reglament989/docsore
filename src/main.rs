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
    document.id = store.id()?;

    let key1 = store.collection("test")?.put(&mut document)?;

    let mut document: DocsoreDocument<Test> = data.clone().into();

    document.id = store.id()?;
    document.make_relation("friends", key1);

    let key2 = store.collection("test")?.put(&mut document)?;

    let mut document: DocsoreDocument<Test> = data.clone().into();
    document.id = store.id()?;
    document.make_relation("friends", key1);
    document.make_relation("friends", key2);
    let key3 = store.collection("test")?.put(&mut document)?;

    dbg!(store.collection("test")?.get::<Test>(&key3)?);

    let index = seahash::hash(document.body.name.as_bytes())
        .to_ne_bytes()
        .to_vec();

    store
        .collection("test")?
        .index(&document.id, vec![index.clone()])?;

    dbg!(store.collection("test")?.count()?);
    dbg!(store
        .collection("test")?
        .search::<Test>(adapter::Filter::Index(index))?);
    Ok(())
}
