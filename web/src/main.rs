use dioxus::logger::tracing::{error, info};
use dioxus::prelude::*;
use dioxus_fullstack::prelude::*;
use api::bingo::BingoGame;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const GRID_SIZE: usize = 8;

fn main() {
    #[cfg(feature = "web")]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                launch_server(App).await;
            });
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Bingo {}
    }
}

#[server]
async fn get_bingo() -> Result<BingoGame, ServerFnError> {
    let json_data = include_str!("../assets/bingo-gf-2025.json");
    Ok(BingoGame::new(
        json_data,
        8,
    )?)
}

#[cfg(feature = "server")]
async fn launch_server(component: fn() -> Element) {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus::cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::default(), App)
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}

#[component]
pub fn Bingo() -> Element {

    let bingo = use_resource(|| async move {
       get_bingo()
           .await
    });

    rsx! {
        match &*bingo.read_unchecked() {
            Some(Ok(bingo_game)) => rsx! {
                BingoGameDiv {
                    bingo_game: bingo_game.clone()
                }
            },
            Some(Err(err)) => rsx! {
                p {
                    "Could not load bingo: {err}"
                }
            },
            None => rsx! {
                p {
                    "Loading..."
                }
            }
        }

    }
}

#[component]
fn BingoGameDiv(bingo_game: BingoGame) -> Element {
    let bingo_signal = use_signal(|| {
        bingo_game
    });

    rsx!(
        div {
            id: "bingo",
            div {
                id: "bingo-grid",

                for row in 0..GRID_SIZE {
                    for col in 0..GRID_SIZE {
                        BingoCell {
                            bingo_game: bingo_signal,
                            row,
                            col,
                        }
                    }
                }
            }
        }
    )
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
                        bingo_game.with_mut(|ref mut game| {
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
                    bingo_game.with_mut(|ref mut game| {
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
                        bingo_game.with_mut(|ref mut game| {
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
                    bingo_game.with_mut(|ref mut game| {
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