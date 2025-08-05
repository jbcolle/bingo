use std::collections::HashMap;
use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BingoItem {
    pub name: String,
    pub done: bool
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BingoGame {
    items: Vec<BingoItem>,
    grid_size: usize
}

impl BingoGame {
    pub fn new(json_data: &str, grid_size: usize) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, bool> = serde_json::from_str(json_data)?;

        let mut items: Vec<BingoItem> = data
            .iter()
            .map(|(text, completed)| BingoItem {
                name: text.to_owned(),
                done: *completed,
            })
            .collect();

        items.truncate((grid_size * grid_size) as usize);

        Ok(BingoGame {
            items,
            grid_size
        })
    }

    pub fn get_item(&self, row: usize, col: usize) -> Option<&BingoItem> {
        let index = (row * self.grid_size) + col;
        self.items.get(index)
    }

    pub fn set_item_completed(&mut self, row: usize, col: usize, completed: bool) -> anyhow::Result<()> {
        let index = (row * self.grid_size) + col;
        let Some(item) = self.items.get_mut(index) else {
            bail!("Could not get bingo item at row {row} and col {col}")
        };
        item.done = completed;
        Ok(())
    }

    fn write_to_json(&self) -> String {
        let mut data: HashMap<String, bool> = HashMap::new();
        for item in &self.items {
            data.insert(item.name.clone(), item.done);
        }
        serde_json::to_string(&data).unwrap()

    }
}