use std::collections::HashMap;
use anyhow::bail;
use dioxus::logger::tracing::{error, info, warn};
use dioxus::prelude::*;
use serde::Deserialize;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const GRID_SIZE: usize = 8;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Bingo {}

    }
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
struct BingoItem {
    name: String,
    done: bool
}

#[derive(Clone, PartialEq, Debug)]
struct BingoGame {
    items: Vec<BingoItem>,
    grid_size: usize
}

impl BingoGame {
    fn new(json_data: &str, grid_size: usize) -> Result<Self, serde_json::Error> {
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

    fn get_item(&self, row: usize, col: usize) -> Option<&BingoItem> {
        let index = (row * self.grid_size) + col;
        self.items.get(index)
    }

    fn set_item_completed(&mut self, row: usize, col: usize, completed: bool) -> anyhow::Result<()> {
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

#[component]
pub fn Bingo() -> Element {

    let bingo = use_signal(|| {
        let json_data = include_str!("../assets/bingo-gf-2025.json");
        BingoGame::new(json_data, GRID_SIZE).expect("Could not load bingo")
    });

    rsx! {
        div {
            id: "bingo",
            div {
                id: "bingo-grid",

                for row in 0..GRID_SIZE {
                    for col in 0..GRID_SIZE {
                        BingoCell {
                            bingo_game: bingo,
                            row,
                            col,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BingoCell(bingo_game: Signal<BingoGame>, row: usize, col: usize) -> Element {

    let item = bingo_game.read().get_item(row, col).expect("Could not get bingo item").clone();

    let mut long_press_timer = use_signal(|| None::<gloo::timers::callback::Timeout>);
    let mut long_press_triggered = use_signal(|| false);

    rsx!(
        button {
            class: if item.done { "bingo-cell completed" } else { "bingo-cell" },

            // Mouse events
            onmousedown: move |_| {
                long_press_triggered.set(false);
                if item.done {
                    let timer = gloo::timers::callback::Timeout::new(800, move || {
                        info!("Long press detected - uncompleting cell at ({}, {})", row, col);
                        long_press_triggered.set(true);
                        bingo_game.with_mut(|game| {
                            if let Err(e) = game.set_item_completed(row, col, false) {
                                error!("Couldn't uncomplete item: {}", e);
                            }
                        });
                    });
                    long_press_timer.set(Some(timer));
                }
            },

            onmouseup: move |_| {
                if let Some(timer) = long_press_timer.take() {
                    timer.cancel();
                }

                // Only process regular click if long press wasn't triggered
                if !long_press_triggered() && !item.done {
                    info!("Clicked cell at ({}, {})", row, col);
                    bingo_game.with_mut(|game| {
                        if let Err(e) = game.set_item_completed(row, col, true) {
                            error!("Couldn't complete item: {}", e);
                        }
                    });
                }

                // Reset for next interaction
                long_press_triggered.set(false);
            },

            onmouseleave: move |_| {
                if let Some(timer) = long_press_timer.take() {
                    timer.cancel();
                }
                long_press_triggered.set(false);
            },

            // Touch events for mobile
            ontouchstart: move |_| {
                long_press_triggered.set(false);
                if item.done {
                    let timer = gloo::timers::callback::Timeout::new(800, move || {
                        info!("Long press detected (touch) - uncompleting cell at ({}, {})", row, col);
                        long_press_triggered.set(true);
                        bingo_game.with_mut(|game| {
                            if let Err(e) = game.set_item_completed(row, col, false) {
                                error!("Couldn't uncomplete item: {}", e);
                            }
                        });
                    });
                    long_press_timer.set(Some(timer));
                }
            },

            ontouchend: move |_| {
                if let Some(timer) = long_press_timer.take() {
                    timer.cancel();
                }

                // Only process regular tap if long press wasn't triggered
                if !long_press_triggered() && !item.done {
                    info!("Tapped cell at ({}, {})", row, col);
                    bingo_game.with_mut(|game| {
                        if let Err(e) = game.set_item_completed(row, col, true) {
                            error!("Couldn't complete item: {}", e);
                        }
                    });
                }

                // Reset for next interaction
                long_press_triggered.set(false);
            },

            "{item.name}"
        }
    )
}