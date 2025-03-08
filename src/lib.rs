use gloo::utils::{document, window};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use web_sys::{FileReader, HtmlInputElement};
use wasm_bindgen::JsCast;
use log::Level;
use console_log;
use wasm_bindgen_futures::spawn_local;
mod image;
use image::{ImageData, SearchResults};

// Main application state
#[derive(Default)]
struct SubimageSearch {
    main_image: Option<String>,
    search_image: Option<String>,
    processing: bool, // Track if processing is in progress
    result: Option<SearchResults>, // Store result message
    progress: f32, // Track progress of image processing (0.0 to 1.0)
    max_mse: f64, // Maximum mean squared error threshold
    max_results: u16, // Maximum number of search results
}

// Application messages
enum Msg {
    MainImageLoaded(String),
    SearchImageLoaded(String),
    ProcessImages,
    UpdateProgress(f32),
    ProcessingComplete(Option<SearchResults>), // Result message from processing
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
                            let result = main_img_data.find_subimage(
                                &search_img_data,
                                move |progress| link_cloned.send_message(Msg::UpdateProgress(progress)),
                                max_mse,
                                max_results,
                            ).await;
                            link.send_message(Msg::ProcessingComplete(Some(result)));
                        }
                        Err(err) => {
                            log::error!("Error loading images: {}", err);
                            window().alert_with_message(&format!("Error loading images: {}", err)).unwrap();
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
        let main_onchange = self.handle_file_upload(ctx, Msg::MainImageLoaded);
        let search_onchange = self.handle_file_upload(ctx, Msg::SearchImageLoaded);
        
        // Determine if the Process button should be enabled
        let both_images_loaded = self.main_image.is_some() && self.search_image.is_some();
        let process_button_class = if both_images_loaded && !self.processing {
            "process-button ready" 
        } else { 
            "process-button disabled" 
        };
        
        // Handle Process button click
        let on_process = ctx.link().callback(|_| Msg::ProcessImages);
        
        // Handle max_mse input change
        let on_max_mse_change = ctx.link().callback(|e: InputEvent| {
            let value_str = e.target_dyn_into::<HtmlInputElement>().unwrap().value();
            let value = value_str.parse::<f64>().unwrap();  // safe, because input type is number
            Msg::UpdateMaxMse(value)
        });

        // Handle max_results input change
        let on_max_results_change = ctx.link().callback(|e: InputEvent| {
            let value_str = e.target_dyn_into::<HtmlInputElement>().unwrap().value();
            let value = value_str.parse::<u16>().unwrap(); // safe, because input type is number
            Msg::UpdateMaxResults(value)
        });

        // Format progress percentage
        let progress_percent = (self.progress * 100.0) as u32;

        html! {
            <div class="container">
                <h1>{"Subimage Search"}</h1>
                {
                    if self.result.is_none() {
                        html! {
                            <>
                                <h2>{"Images"}</h2>
                                <div class="image-inputs">
                                    {image_input("Main Image", "mainImageInput", "mainImagePreview", main_onchange, &self.main_image, None)}
                                    {image_input("Image to search", "searchImageInput", "searchImagePreview", search_onchange, &self.search_image, Some(image_search_help()))}
                                </div>

                                <h2>{"Settings"}</h2>
                                <div class="settings">
                                    <label class="settings-item">
                                        <h3>{"Maximum difference (%)"}</h3>
                                        <input
                                            type="number"
                                            id="maxMseInput"
                                            value={(self.max_mse * 100.0).to_string()}
                                            oninput={on_max_mse_change}
                                            disabled={self.processing}
                                            step="0.1"
                                            min="0"
                                            max="100"
                                        />
                                        <span class="unit">{"%"}</span>
                                        <ul class="settings-hint">
                                            <li><a href="https://en.wikipedia.org/wiki/Mean_squared_error" target="_blank">{"Mean squared error"}</a>{" threshold"}</li>
                                            <li>{"0% - exact match"}</li>
                                            <li>{"100% - any difference"}</li>
                                            <li>{"Alpha channel is also considered as a color component."}</li>
                                            <li>{"Low values usually cause faster search due to optimizations."}</li>
                                        </ul>
                                    </label>
                                    <label class="settings-item">
                                        <h3>{"Maximum number of results"}</h3>
                                        <input
                                            type="number"
                                            id="maxResultsInput"
                                            value={self.max_results.to_string()}
                                            oninput={on_max_results_change}
                                            disabled={self.processing}
                                            step="1"
                                            min="1"
                                            max="100"
                                        />
                                        <ul class="settings-hint">
                                            <li>{"Maximum number of search results to display"}</li>
                                            <li>{"When there are more matches, the most relevant are shown."}</li>
                                        </ul>
                                    </label>
                                </div>
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
                        let on_edit = ctx.link().callback(|_| Msg::NewSearch);
                        html! {
                            <div class="search-summary">
                            <h2>{"Search summary"}</h2>
                                <div class="search-info">
                                    <div class="search-image-preview">
                                        <h3>{"Searched subimage"}</h3>
                                        <img
                                            src={self.search_image.clone().unwrap_or_default()}
                                            alt="Subimage that was searched"
                                        />
                                    </div>
                                    <div class="settings-summary">
                                        <h3>{"Search Settings"}</h3>
                                        <span class="setting">{"Maximum difference: "}<strong>{format!("{:.1}%", self.max_mse * 100.0)}</strong></span>
                                        <span class="setting">{"Maximum results: "}<strong>{self.max_results}</strong></span>
                                    </div>
                                    <button class="edit-button" onclick={on_edit}>{"New Search"}</button>
                                </div>
                            </div>
                        }
                    }
                }

                <div id="results">
                    {
                        if let Some(result) = &self.result {
                            html! {
                                <div class="result-container">
                                    <h2>{"Search results"}</h2>
                                    <div class="result-message">
                                        <h3>{if result.has_overflown() {
                                            format!("Found many matches, showing {} most relevant", result.get_matches().len())
                                        } else {
                                            format!("Found {} matches", result.get_matches().len())
                                        }}</h3>
                                    </div>
                                    <div class="main-image-container">
                                        <img
                                            src={self.main_image.clone().unwrap_or_default()}
                                            alt="Main image with matches"
                                            class="result-main-image"
                                        />
                                        {
                                            result.get_matches().iter().enumerate().map(|(i, m)| {
                                                let x_percent = (m.x as f64 / result.get_main_width() as f64 * 100.0) as f64;
                                                let y_percent = (m.y as f64 / result.get_main_height() as f64 * 100.0) as f64;
                                                let width_percent = (result.get_template_width() as f64 / result.get_main_width() as f64 * 100.0) as f64;
                                                let height_percent = (result.get_template_height() as f64 / result.get_main_height() as f64 * 100.0) as f64;

                                                html! {
                                                    <div
                                                        class="match-overlay"
                                                        style={format!(
                                                            "left: {}%; top: {}%; width: {}%; height: {}%",
                                                            x_percent, y_percent, width_percent, height_percent
                                                        )}
                                                        title={format!("#{} | MSE: {:.4}", i+1, m.mse)}
                                                        data-match-id={i.to_string()}
                                                    />
                                                }
                                            }).collect::<Html>()
                                        }
                                    </div>
                                    <ol class="matches-list">
                                        {
                                            result.get_matches().iter().enumerate().map(|(i, m)| {
                                                html! {
                                                    <li class="match-item" data-match-id={i.to_string()}>
                                                        {format!("Match at ({}, {}) - MSE: {:.4}%", m.x, m.y, m.mse*100.0)}
                                                    </li>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </ol>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>
        }
    }
}

fn image_search_help() -> Html{
     html! {
        <ul class="image-hint">
            <li><strong>{"Orientation"}</strong>{" has to be the same as in main image."}</li>
            <li><strong>{"Scale"}</strong>{" has to be the same as in main image."}</li>
            <li><strong>{"Compression artifacts and blur caused by scaling up"}</strong>{" can be handled by increasing the maximum difference."}</li>
            <li><strong>{"Alpha channel"}</strong>{" cannot be used as a wildcard. It is considered as a color component. If you don't know the consequences, you probably don't want to search an image with significant transparency."}</li>
        </ul>
    }
}

fn image_input(label: &str, input_id: &str, preview_id: &str, onchange: Callback<Event>, image: &Option<String>, help: Option<Html>) -> Html {
    html! {
        <label class="image-input">
            <h3>{label}</h3>
            <input
                type="file"
                id={input_id.to_string()}
                accept="image/*"
                onchange={onchange}
            />
            {
                if image.is_none() {
                    html! {
                        <p>{"Click here to select an image"}</p>
                    }
                }else {
                    html! {
                        <img
                            id={preview_id.to_string()}
                            class="preview"
                            alt={format!("{} preview", label)}
                            src={image.clone().unwrap_or_default()}
                        />
                    }
                }
            }
            {help.unwrap_or_default()}
        </label>
    }
}

// Helper methods for SubimageSearch
impl SubimageSearch {
    fn handle_file_upload(&self, ctx: &Context<Self>, msg_creator: fn(String) -> Msg) -> Callback<Event> {
        let link = ctx.link().clone();
        
        Callback::from(move |e: Event| {
            let target = e.target().unwrap();
            let input: HtmlInputElement = target.dyn_into().unwrap();
            
            if let Some(file_list) = input.files() {
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
    let image: web_sys::HtmlImageElement = document().get_element_by_id(image_id).unwrap().dyn_into().unwrap();
    ImageData::from_image(&image)
}

// Starting the Yew application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<SubimageSearch>::new().render();
    Ok(())
}
