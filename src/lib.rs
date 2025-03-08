use wasm_bindgen::prelude::*;
use yew::prelude::*;
use web_sys::{FileReader, HtmlInputElement};
use wasm_bindgen::JsCast;
use log::Level;
use console_log;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;


struct ImageData {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA pixel data
}

// Main application state
#[derive(Default)]
struct SubimageSearch {
    main_image: Option<String>,
    search_image: Option<String>,
    processing: bool, // Track if processing is in progress
    result: Option<String>, // Store result message
}

// Application messages
enum Msg {
    MainImageLoaded(String),
    SearchImageLoaded(String),
    ProcessImages,
    ProcessingComplete(Option<String>), // Result message from processing
}

impl Component for SubimageSearch {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        console_log::init_with_level(Level::Debug).expect("error initializing log");
        log::info!("Subimage Search Application Initialized with Yew");
        Self::default()
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

                // Launch async image processing
                let link = ctx.link().clone();
                spawn_local(async move {
                    match load_images_for_processing().await {
                        Ok((main_img_data, search_img_data)) => {
                            log::info!("Images loaded successfully");
                            // Images loaded successfully - now you can process them
                            let result = process_images(main_img_data, search_img_data);
                            link.send_message(Msg::ProcessingComplete(Some(result)));
                        }
                        Err(err) => {
                            log::error!("Error loading images: {}", err);
                            link.send_message(Msg::ProcessingComplete(Some(format!("Error: {}", err))));
                        }
                    }
                });

                true
            }
            Msg::ProcessingComplete(result) => {
                self.processing = false;
                self.result = result;
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
                </div>
                
                <div id="results" class="results">
                    {
                        if let Some(result) = &self.result {
                            html! {
                                <div class="result-message">
                                    { result }
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
    let document = web_sys::window().unwrap().document().unwrap();
    let image: web_sys::HtmlImageElement = document.get_element_by_id(image_id).unwrap().dyn_into().unwrap();
    let canvas: web_sys::HtmlCanvasElement = document.create_element("canvas").map_err(|e| format!("error creating canvas: {:?}", e))?.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let ctx: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").map_err(|e| format!("error getting 2d context: {:?}", e))?.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let width = image.natural_width();
    let height = image.natural_height();

    // Set canvas size to match image
    canvas.set_width(width);
    canvas.set_height(height);

    ctx.draw_image_with_html_image_element(&image, 0.0, 0.0).unwrap();

    //Ok(ctx)
    let image_data = ctx.get_image_data(0.0, 0.0, width.into(), height.into()).unwrap();
    //log::info!("image data ({}): {:?}", image_data.data().len(), image_data.data());
    Ok(ImageData {
        width,
        height,
        pixels: image_data.data().to_vec(),
    })
}


fn get_pixel(image: &ImageData, x: u32, y: u32) -> &[u8] {
    let index = (y * image.width + x) as usize * 4;
    let data: &[u8] = &image.pixels;
    &data[index..index+4]
}
fn get_pixels(image: &ImageData, x: u32, y: u32, count: usize) -> &[u8] {
    let index = (y * image.width + x) as usize * 4;
    &image.pixels[index..index + 4 * count]
}

/*
subpixel: 8b
square error: 16b
resolution like 1920x1080 needs additional 21b, i.e., 37b in total, so u32 is not enough
*/
type TSE = u64;


fn process_images(main_image: ImageData, search_image: ImageData) -> String {
    let square_errors_divisor = search_image.width * search_image.height * 4;
    // y comes first because of memory locality
    for y in 0..(main_image.height - search_image.height) {
        log::info!("Checking line {}", y);
        for x in 0..(main_image.width - search_image.width) {            
            let mut sse: TSE = 0;
            for dy in 0..search_image.height {
                /*for dx in 0..search_image.width {
                    let main_pixel = get_pixel(&main_image, x + dx, y + dy);
                    let search_pixel = get_pixel(&search_image, dx, dy);
                    let square_errors: TSE = main_pixel.iter().zip(search_pixel).map(|(m, s)|
                        ((m-s) as i32).pow(2) as TSE
                    ).sum();
                    //log::info!("Square errors for {:?} and {:?}: {}", main_pixel, search_pixel,  square_errors);
                    sse += square_errors;
                }*/
                let main_pixels = get_pixels(&main_image, x, y + dy, search_image.width as usize);
                let search_pixels = get_pixels(&search_image, 0, dy, search_image.width as usize);
                let square_errors: TSE = main_pixels.iter().zip(search_pixels).map(|(m, s)|
                    ((m-s) as i32).pow(2) as TSE
                ).sum();
                sse += square_errors;
            }
            
            let mse: f64 = (sse as f64) / (square_errors_divisor as f64) / (65536.0);
            if mse < 0.01 {
                log::info!("pos ({}, {}) ({} pxs)", x, y, search_image.width * search_image.height);
                log::info!("sum of square errors: {} / MSE: {}", sse, mse);
            }
        }
    }
    "Images loaded for processing".to_string()
}

// Starting the Yew application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<SubimageSearch>::new().render();
    Ok(())
}
