use gloo::utils::window;
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
}

// Application messages
enum Msg {
    MainImageLoaded(String),
    SearchImageLoaded(String),
    ProcessImages,
    UpdateProgress(f32),
    ProcessingComplete(Option<SearchResults>), // Result message from processing
    UpdateMaxMse(f64),
}

impl Component for SubimageSearch {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        console_log::init_with_level(Level::Debug).expect("error initializing log");
        log::info!("Subimage Search Application Initialized with Yew");
        Self {
            max_mse: 0.01,
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

        // Format progress percentage
        let progress_percent = (self.progress * 100.0) as u32;

        html! {
            <div class="container">
                <h1>{"Subimage Search"}</h1>
                
                <div class="image-inputs">
                    <div>
                        <h2>{"Main Image"}</h2>
                        <input 
                            type="file" 
                            id="mainImageInput" 
                            accept="image/*" 
                            onchange={main_onchange}
                        />
                    </div>
                    
                    <div>
                        <h2>{"Search Image"}</h2>
                        <input 
                            type="file" 
                            id="searchImageInput" 
                            accept="image/*" 
                            onchange={search_onchange}
                        />
                    </div>
                </div>
                
                <div class="image-preview">
                    <div class="preview-container">
                        <h3>{"Main Image Preview"}</h3>
                        <img 
                            id="mainImagePreview" 
                            class="preview" 
                            alt="Main image preview" 
                            src={self.main_image.clone().unwrap_or_default()} 
                        />
                    </div>
                    <div class="preview-container">
                        <h3>{"Search Image Preview"}</h3>
                        <img 
                            id="searchImagePreview" 
                            class="preview" 
                            alt="Search image preview" 
                            src={self.search_image.clone().unwrap_or_default()} 
                        />
                    </div>
                </div>
                
                <div class="action-section">
                    <button 
                        class={process_button_class}
                        onclick={on_process}
                        disabled={!both_images_loaded || self.processing}
                    >
                        {
                            if self.processing {
                                "Processing..."
                            } else {
                                "Process Images"
                            }
                        }
                    </button>

                    {
                        if self.processing {
                            html! {
                                <div class="progress-container">
                                    <progress value={self.progress.to_string()} max="1"></progress>
                                    <span class="progress-text">{format!("{}%", progress_percent)}</span>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                
                <div class="settings">
                    <label for="maxMseInput">{"Maximum difference measured by MSE (%):"}</label>
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
                </div>

                <div id="results" class="results">
                    {
                        if let Some(result) = &self.result {
                            html! {
                                <div class="result-message">
                                    { format!("{:?}", result) }
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
    let image: web_sys::HtmlImageElement = web_sys::window().unwrap().document().unwrap().get_element_by_id(image_id).unwrap().dyn_into().unwrap();
    ImageData::from_image(&image)
}

// Starting the Yew application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<SubimageSearch>::new().render();
    Ok(())
}
