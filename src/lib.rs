use wasm_bindgen::prelude::*;
use yew::prelude::*;
use web_sys::{File, FileReader, HtmlInputElement};
use wasm_bindgen::JsCast;
use log::Level;
use console_log;

// Main application state
#[derive(Default)]
struct SubimageSearch {
    main_image: Option<String>,
    search_image: Option<String>,
}

// Application messages
enum Msg {
    MainImageLoaded(String),
    SearchImageLoaded(String),
}

impl Component for SubimageSearch {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        console_log::init_with_level(Level::Debug).expect("error initializing log");
        log::info!("Subimage Search Application Initialized with Yew");
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MainImageLoaded(data_url) => {
                self.main_image = Some(data_url);
                true
            }
            Msg::SearchImageLoaded(data_url) => {
                self.search_image = Some(data_url);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let main_onchange = self.handle_file_upload(ctx, Msg::MainImageLoaded);
        let search_onchange = self.handle_file_upload(ctx, Msg::SearchImageLoaded);
        
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
                
                <div id="results" class="results">
                    // Results will be displayed here
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

// Starting the Yew application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<SubimageSearch>::new().render();
    Ok(())
}
