use std::{collections::BTreeMap, fmt::Debug};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type HashIndex = Vec<u8>;

pub struct Store {
    pub sled: sled::Db,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            sled: sled::open("./docsore.db").expect("Cant open store in current dir"),
        }
    }
}

impl Store {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let sled = sled::open(path)?;
        Ok(Store { sled })
    }

    pub fn generate_id(&self) -> anyhow::Result<u64> {
        Ok(self.sled.generate_id()?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DocsoreDocument<S: Document> {
    pub relations: BTreeMap<String, Vec<[u8; 8]>>,
    pub id: u64,
    pub body: S,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relation(String, Vec<u8>);

impl<D: Document> From<&D> for Relation {
    fn from(doc: &D) -> Self {
        Self(D::collection().to_string(), doc.id())
    }
}

#[derive(Debug)]
pub enum Filter {
    Index(HashIndex),
}

// pub enum Indexes {
// HashIndex
// }

pub trait Document: Serialize + Sized {
    const RELATION_KEY: &'static [u8] = &[0, 1];
    const INDEXES_KEY: &'static [u8] = &[0, 2];

    fn collection() -> &'static str;
    fn indexes(&self) -> Vec<HashIndex>;
    fn id(&self) -> Vec<u8>;

    fn find(store: &Store, filter: Filter) -> anyhow::Result<Self>
    where
        Self: DeserializeOwned,
    {
        match filter {
            Filter::Index(idx) => {
                let mut key: Vec<u8> = Self::collection().as_bytes().to_vec();
                key.extend_from_slice(Self::INDEXES_KEY);
                let tree = store.sled.open_tree(&key)?;
                let key_by_index = tree.get(idx)?.ok_or(anyhow::anyhow!("Indexes not found"))?;
                let tree = store.sled.open_tree(Self::collection())?;
                let doc = tree
                    .get(&key_by_index)?
                    .ok_or(anyhow::anyhow!("Document by index not found"))?;
                Ok(bincode::deserialize::<Self>(&doc)?)
            }
        }
    }

    fn save(&self, store: &Store) -> anyhow::Result<()> {
        let tree = store.sled.open_tree(Self::collection())?;
        tree.insert(self.id(), bincode::serialize(self)?)?;
        Ok(())
    }

    fn make_relation(&self, store: &Store, relations: Vec<Relation>) -> anyhow::Result<()> {
        let tree = store.sled.open_tree(Self::collection())?;
        let mut key = self.id();
        key.append(&mut Self::RELATION_KEY.to_vec());
        let mut rel_ids =
            bincode::deserialize::<Vec<Relation>>(&tree.get(&key)?.unwrap_or_default())
                .unwrap_or_default();
        for relation in relations {
            rel_ids.push(relation);
        }
        tree.insert(key, bincode::serialize(&rel_ids)?)?;
        Ok(())
    }

    fn relation<D: Document + DeserializeOwned>(&self, store: &Store) -> anyhow::Result<Vec<D>> {
        let tree = store.sled.open_tree(Self::collection())?;
        let mut key = self.id();
        key.append(&mut Self::RELATION_KEY.to_vec());
        let raw = tree.get(key)?;
        match raw {
            Some(doc) => {
                let relations: Vec<Relation> = bincode::deserialize::<Vec<Relation>>(&doc)?
                    .into_iter()
                    .filter(|r| r.0 == D::collection())
                    .collect();
                let mut docs = vec![];
                for relation in relations {
                    let tree = store.sled.open_tree(relation.0)?;
                    let raw = tree.get(relation.1)?;
                    match raw {
                        Some(doc) => docs.push(bincode::deserialize::<D>(&doc)?),
                        None => continue,
                    }
                }
                Ok(docs)
            }
            None => Err(anyhow::anyhow!("Relations not found")),
        }
    }

    fn index(&self, store: &Store) -> anyhow::Result<()>
    where
        Self: DeserializeOwned,
    {
        let mut key: Vec<u8> = Self::collection().as_bytes().to_vec();
        key.extend_from_slice(Self::INDEXES_KEY);
        let tree = store.sled.open_tree(key)?;
        let indexes = self.indexes();

        for index in indexes {
            tree.insert(index, self.id())?;
        }
        Ok(())
    }

    fn reindex(&self, store: &Store) -> anyhow::Result<()>
    where
        Self: DeserializeOwned,
    {
        let mut batch = sled::Batch::default();
        let tree = store.sled.open_tree(Self::collection())?;
        for raw in tree.iter() {
            let (key, doc) = raw?;
            let doc = bincode::deserialize::<Self>(&doc).unwrap();
            let new_indexes = doc.indexes();
            for idx in new_indexes {
                batch.insert(idx, &key);
            }
        }
        let mut key: Vec<u8> = Self::collection().as_bytes().to_vec();
        key.extend_from_slice(Self::INDEXES_KEY);
        let tree = store.sled.open_tree(key)?;
        tree.clear()?;
        tree.apply_batch(batch)?;
        Ok(())
    }
}
