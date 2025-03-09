use gloo::utils::{document, window};
use log::Level;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileList;
use web_sys::FileReader;
use yew::prelude::*;
mod image;
use image::{ImageData, SearchResults};

mod components {
    pub mod image_input;
    pub mod search_params;
    pub mod search_results;
    pub mod search_summary;
}

use components::search_params::SearchParams;
use components::search_results::SearchError;
use components::search_summary::SearchSummary;

// Main application state
#[derive(Default)]
struct SubimageSearch {
    main_image: Option<String>,
    search_image: Option<String>,
    processing: bool, // Track if processing is in progress
    result: Option<Result<SearchResults, String>>, // Store result message
    progress: f32,    // Track progress of image processing (0.0 to 1.0)
    max_mse: f64,     // Maximum mean squared error threshold
    max_results: u16, // Maximum number of search results
}

// Application messages
enum Msg {
    MainImageLoaded(String),
    SearchImageLoaded(String),
    ProcessImages,
    UpdateProgress(f32),
    ProcessingComplete(Option<Result<SearchResults, String>>), // Result message from processing
    UpdateMaxMse(f64),
    UpdateMaxResults(u16), // Message to update max_results
    NewSearch,
}

impl Component for SubimageSearch {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        console_log::init_with_level(Level::Debug).expect("error initializing log");
        log::info!("Subimage Search Application Initialized with Yew");
        Self {
            max_mse: 0.01,
            max_results: 10,
            ..Self::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MainImageLoaded(data_url) => {
                self.main_image = Some(data_url);
                true
            }
            Msg::SearchImageLoaded(data_url) => {
                self.search_image = Some(data_url);
                true
            }
            Msg::ProcessImages => {
                log::info!("Starting image processing...");
                self.processing = true;
                self.result = None;
                self.progress = 0.0; // Reset progress

                // Launch async image processing
                let link = ctx.link().clone();
                let max_mse = self.max_mse;
                let max_results = self.max_results;
                spawn_local(async move {
                    match load_images_for_processing().await {
                        Ok((main_img_data, search_img_data)) => {
                            log::info!("Images loaded successfully");
                            // Images loaded successfully - now you can process them
                            let link_cloned = link.clone();
                            let result = main_img_data
                                .find_subimage(
                                    &search_img_data,
                                    move |progress| {
                                        link_cloned.send_message(Msg::UpdateProgress(progress))
                                    },
                                    max_mse,
                                    max_results,
                                )
                                .await;
                            link.send_message(Msg::ProcessingComplete(Some(result)));
                        }
                        Err(err) => {
                            log::error!("Error loading images: {}", err);
                            window()
                                .alert_with_message(&format!("Error loading images: {}", err))
                                .unwrap();
                            link.send_message(Msg::ProcessingComplete(None));
                        }
                    }
                });

                true
            }
            Msg::UpdateProgress(progress) => {
                self.progress = progress;
                true
            }
            Msg::ProcessingComplete(result) => {
                self.processing = false;
                self.result = result;
                self.progress = 1.0; // Ensure progress is complete
                true
            }
            Msg::UpdateMaxMse(new_max_mse_percent) => {
                self.max_mse = new_max_mse_percent / 100.0;
                true
            }
            Msg::UpdateMaxResults(new_max_results) => {
                self.max_results = new_max_results;
                true
            }
            Msg::NewSearch => {
                self.result = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Determine if the Process button should be enabled
        let both_images_loaded = self.main_image.is_some() && self.search_image.is_some();
        let process_button_class = if both_images_loaded && !self.processing {
            "process-button ready"
        } else {
            "process-button disabled"
        };

        // Handle Process button click
        let on_process = ctx.link().callback(|_| Msg::ProcessImages);

        // Format progress percentage
        let progress_percent = (self.progress * 100.0) as u32;

        html! {
            <div class="container">
                <h1>{"Subimage Search"}</h1>
                {
                    if self.result.is_none() {
                        html! {
                            <>

                                <SearchParams
                                    max_mse={self.max_mse}
                                    max_results={self.max_results}
                                    main_image={self.main_image.clone()}
                                    search_image={self.search_image.clone()}
                                    disabled={self.processing}
                                    on_max_mse_change={ctx.link().callback(Msg::UpdateMaxMse)}
                                    on_max_results_change={ctx.link().callback(Msg::UpdateMaxResults)}
                                    on_main_image_upload={self.handle_file_upload(ctx, Msg::MainImageLoaded)}
                                    on_search_image_upload={self.handle_file_upload(ctx, Msg::SearchImageLoaded)}
                                />

                                <div class="action-section">
                                    <button
                                        class={process_button_class}
                                        onclick={on_process}
                                        disabled={!both_images_loaded || self.processing}
                                    >
                                        {
                                            if self.processing {
                                                "Searching..."
                                            } else {
                                                "Search subimage"
                                            }
                                        }
                                    </button>

                                    {
                                        if self.processing {
                                            html! {
                                                <div class="progress-container">
                                                    <progress value={self.progress.to_string()} max="1"></progress>
                                                    <span class="progress-text">{format!("{}%", progress_percent)}</span>
                                                    <div class="progress-hint">
                                                        {"Progress indicator might be sometimes inconsistent due to various optimizations that apply on some part of the image more than on others."}
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>

                            </>
                        }
                    } else {
                        html! {
                            <SearchSummary
                                search_image={self.search_image.clone().unwrap_or_default()}
                                max_mse={self.max_mse}
                                max_results={self.max_results}
                                on_new_search={ctx.link().callback(|_| Msg::NewSearch)}
                            />
                        }
                    }
                }

                <div id="results">
                    {
                        if let Some(result) = &self.result {
                            match result {
                                Ok(search_results) => {
                                    html! {
                                        <div class="result-container">
                                            <h2>{"Search results"}</h2>
                                            <div class="result-message">
                                                <h3>{if search_results.has_overflown() {
                                                    format!("Found many matches, showing {} most relevant", search_results.get_matches().len())
                                                } else if search_results.get_matches().is_empty() {
                                                    "No matches found".to_string()
                                                } else {
                                                    format!("Found {} matches", search_results.get_matches().len())
                                                }}</h3>
                                            </div>
                                            <div class="main-image-container">
                                                <img
                                                    src={self.main_image.clone().unwrap_or_default()}
                                                    alt="Main image with matches"
                                                    class="result-main-image"
                                                />
                                                {
                                                    search_results.get_matches().iter().enumerate().map(|(i, m)| {
                                                        let x_percent = m.x as f64 / search_results.get_main_width() as f64 * 100.0;
                                                        let y_percent = m.y as f64 / search_results.get_main_height() as f64 * 100.0;
                                                        let width_percent = search_results.get_template_width() as f64 / search_results.get_main_width() as f64 * 100.0;
                                                        let height_percent = search_results.get_template_height() as f64 / search_results.get_main_height() as f64 * 100.0;

                                                        html! {
                                                            <div
                                                                class="match-overlay"
                                                                style={format!(
                                                                    "left: {}%; top: {}%; width: {}%; height: {}%",
                                                                    x_percent, y_percent, width_percent, height_percent
                                                                )}
                                                                title={format!("#{} | MSE: {:.4}", i+1, m.get_mse(search_results.get_squared_errors_divisor()))}
                                                                data-match-id={i.to_string()}
                                                            />
                                                        }
                                                    }).collect::<Html>()
                                                }
                                            </div>
                                            <ol class="matches-list">
                                                {
                                                    search_results.get_matches().iter().enumerate().map(|(i, m)| {
                                                        html! {
                                                            <li class="match-item" data-match-id={i.to_string()}>
                                                                {format!("Match at ({}, {}) - MSE: {:.4}%",
                                                                    m.x,
                                                                    m.y,
                                                                    m.get_mse(search_results.get_squared_errors_divisor())*100.0
                                                                )}
                                                            </li>
                                                        }
                                                    }).collect::<Html>()
                                                }
                                            </ol>
                                        </div>
                                    }
                                }
                                Err(error_message) => {
                                    html! {
                                        <SearchError message={error_message.clone()} />
                                    }
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>

                <footer class="footer">
                    <p>{"Source code available on "}
                        <a href="https://github.com/v6ak/subimage-search" target="_blank">
                            {"GitHub"}
                        </a>
                    </p>
                </footer>
            </div>
        }
    }
}

// Helper methods for SubimageSearch
impl SubimageSearch {
    fn handle_file_upload(
        &self,
        ctx: &Context<Self>,
        msg_creator: fn(String) -> Msg,
    ) -> Callback<FileList> {
        let link = ctx.link().clone();

        Callback::from(move |file_list: FileList| {
            if let Some(file) = file_list.get(0) {
                let file_reader = FileReader::new().unwrap();
                let fr_clone = file_reader.clone();
                let link_clone = link.clone();

                let onload_closure = Closure::wrap(Box::new(move |_: Event| {
                    let result = fr_clone.result().unwrap();
                    if let Some(data_url) = result.as_string() {
                        link_clone.send_message(msg_creator(data_url));
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                file_reader.read_as_data_url(&file).unwrap();
                onload_closure.forget();
            }
        })
    }
}

// Image processing functions
async fn load_images_for_processing() -> Result<(ImageData, ImageData), String> {
    // Create a promise that resolves when both images are loaded
    let main_image_data = load_image_data("mainImagePreview").await?; // main_img_url
    log::info!("main image loaded");
    let search_image_data = load_image_data("searchImagePreview").await?;
    log::info!("search image loaded");

    Ok((main_image_data, search_image_data))
}

// Load a single image and extract its pixel data

async fn load_image_data(image_id: &str) -> Result<ImageData, String> {
    let image: web_sys::HtmlImageElement = document()
        .get_element_by_id(image_id)
        .unwrap()
        .dyn_into()
        .unwrap();
    ImageData::from_image(&image)
}

// Starting the Yew application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<SubimageSearch>::new().render();
    Ok(())
}
