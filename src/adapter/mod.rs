use std::{collections::BTreeMap, time::Instant};

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct Store {
    sled: sled::Db,
}

pub struct Collection<'a> {
    sled: &'a sled::Db,
    name: String,
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

    pub fn collection(&self, name: &str) -> anyhow::Result<Collection> {
        Ok(Collection::new(&self.sled, name))
    }

    pub fn id(&self) -> anyhow::Result<u64> {
        Ok(self.sled.generate_id()?)
    }
}

pub type HashIndex = Vec<u8>;
pub type Id = [u8; 8];

#[derive(Debug, Deserialize, Serialize)]
pub struct DocsoreDocument<S> {
    relations: BTreeMap<String, Vec<Id>>,
    id: u64,
    body: S,
}

impl<S> DocsoreDocument<S> {
    pub fn make_relation<N: AsRef<str> + Ord>(&mut self, name: N, id: u64) {
        let relations = match self.relations.get_mut(name.as_ref()) {
            Some(relation) => relation,
            None => {
                self.relations.insert(name.as_ref().to_string(), vec![]);
                self.relations.get_mut(name.as_ref()).unwrap()
            }
        };
        relations.push(id.to_ne_bytes());
    }

    pub fn relation<N: AsRef<str> + Ord, D: DeserializeOwned>(
        &self,
        name: N,
        col: &Collection,
    ) -> anyhow::Result<Vec<DocsoreDocument<D>>> {
        match self.relations.get(name.as_ref()) {
            Some(relations) => {
                let mut docs = vec![];
                for relation in relations {
                    docs.push(col.get::<D>(&u64::from_ne_bytes(*relation))?);
                }
                Ok(docs)
            }
            None => todo!(),
        }
    }
}

impl<S: Serialize> From<S> for DocsoreDocument<S> {
    fn from(body: S) -> Self {
        Self {
            id: 0,
            relations: BTreeMap::new(),
            body,
        }
    }
}
pub enum Filter {
    Index(HashIndex),
}

impl<S> DocsoreDocument<S> {
    pub fn new(body: S, relations: Option<BTreeMap<String, Vec<Id>>>) -> Self {
        Self {
            body,
            id: 0,
            relations: relations.unwrap_or_default(),
        }
    }
}

impl<'a> Collection<'a> {
    pub fn new(sled: &'a sled::Db, name: &str) -> Self {
        Self {
            sled,
            name: name.to_owned(),
        }
    }
    pub fn put<S: Serialize>(&self, document: &mut DocsoreDocument<S>) -> anyhow::Result<u64> {
        let start = Instant::now();
        let tree = self.sled.open_tree(&self.name)?;
        let codec = bincode::options();

        document.id = self.sled.generate_id().unwrap();

        tree.insert(&document.id.to_ne_bytes(), codec.serialize(&document)?)?;
        log::info!(
            "Put in tree !{} [Elapsed: {:#?}]",
            &self.name,
            start.elapsed()
        );
        Ok(document.id)
    }

    pub fn get<S: DeserializeOwned>(&self, key: &u64) -> anyhow::Result<DocsoreDocument<S>> {
        let start = Instant::now();
        let tree = self.sled.open_tree(&self.name)?;
        let codec = bincode::options();

        let raw = tree.get(key.to_ne_bytes())?;
        log::info!(
            "Get by key #{} in tree !{} [Elapsed: {:#?}]",
            key,
            &self.name,
            start.elapsed()
        );
        match raw {
            Some(raw) => Ok(codec.deserialize::<DocsoreDocument<S>>(&raw)?),
            None => todo!(),
        }
    }

    pub fn search<S: DeserializeOwned>(
        &self,
        filter: Filter,
    ) -> anyhow::Result<DocsoreDocument<S>> {
        let raw = match filter {
            Filter::Index(idx) => {
                let tree = self.sled.open_tree(&format!("{}_indexes", &self.name))?;
                tree.get(&idx)?
            }
        };

        let codec = bincode::options();
        match raw {
            Some(raw) => Ok(codec.deserialize(&raw)?),
            None => todo!(),
        }
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        let tree = self.sled.open_tree(&self.name)?;
        log::info!("Executing count in tree !{}", &self.name);
        Ok(tree.len())
    }

    pub fn relate(&self, key: &u64, relates: BTreeMap<String, u64>) -> anyhow::Result<()> {
        let tree = self.sled.open_tree(&self.name)?;
        log::info!(
            "Executing relate for #{:#?} in tree !{}",
            &relates,
            &self.name
        );
        let codec = bincode::options();
        tree.merge(&format!("{}_relates", key), codec.serialize(&relates)?)?;
        Ok(())
    }

    pub fn index(&self, key: &u64, indexes: Vec<HashIndex>) -> anyhow::Result<()> {
        let tree = self.sled.open_tree(&format!("{}_indexes", &self.name))?;
        log::info!(
            "Executing indexes for #{:#?} in tree !{}",
            &indexes,
            &self.name
        );
        for index in indexes {
            tree.insert(index, &key.to_ne_bytes())?;
        }
        Ok(())
    }
}
