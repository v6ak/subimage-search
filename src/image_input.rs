use crate::drag_drop::prevent_default;
use crate::drag_drop::setup_drag_blocking_handlers;
use crate::drag_drop::setup_drag_handlers;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{DragEvent, Event, HtmlElement};
use web_sys::{FileList, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ImageInputProps {
    pub label: String,
    pub input_id: String,
    pub preview_id: String,
    pub on_upload: Callback<FileList>,
    pub image: Option<String>,
    pub help: Option<Html>,
    pub disabled: bool,
}

fn on_change(cb: Callback<FileList>) -> Callback<Event> {
    Callback::from(move |e: Event| {
        let target = e.target().unwrap();
        let input: HtmlInputElement = target.dyn_into().unwrap();
        if let Some(files) = input.files() {
            cb.emit(files)
        }
        prevent_default(&e);
    })
}

#[function_component(ImageInput)]
pub fn image_input(props: &ImageInputProps) -> Html {
    let node_ref = use_node_ref();

    // Setup drag & drop handlers
    {
        let node_ref = node_ref.clone();
        let disabled = props.disabled;
        use_effect_with((), move |_| {
            if let Some(element) = node_ref.cast::<HtmlElement>() {
                if disabled {
                    setup_drag_blocking_handlers(&element);
                } else {
                    setup_drag_handlers(&element);
                }
            }
            || ()
        });
    }

    // Handle drop event
    let ondrop = if props.disabled {
        Callback::from(|event: DragEvent| {
            prevent_default(&event);
        })
    } else {
        let on_upload = props.on_upload.clone();
        Callback::from(move |event: DragEvent| {
            prevent_default(&event);
            if let Some(files) = event.data_transfer().and_then(|x| x.files()) {
                if let Some(file) = files.get(0) {
                    let file_reader = web_sys::FileReader::new().unwrap();

                    let on_upload = on_upload.clone();
                    let onload = Closure::wrap(Box::new(move |e: Event| {
                        let target: web_sys::FileReader = e.target().unwrap().dyn_into().unwrap();
                        if let Ok(result) = target.result() {
                            if let Some(data_url) = result.as_string() {
                                on_upload.emit(files.clone());
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    file_reader.read_as_data_url(&file).unwrap();
                    onload.forget();
                }
            }
        })
    };

    html! {
        <label class="image-input" id={format!("{}-container", props.input_id)} ref={node_ref} {ondrop}>
            <h3>{&props.label}</h3>
            <input
                type="file"
                id={props.input_id.clone()}
                accept="image/*"
                onchange={on_change(props.on_upload.clone())}
                disabled={props.disabled}
            />
            {
                if props.image.is_none() {
                    html! {
                        <div class="upload-prompt">
                            <div class="upload-icon">{"+"}</div>
                            <p>{"Click here or drag&drop image"}</p>
                        </div>
                    }
                } else {
                    html! {
                        <img
                            id={props.preview_id.clone()}
                            class="preview"
                            alt={format!("{} preview", props.label)}
                            src={props.image.clone().unwrap_or_default()}
                        />
                    }
                }
            }
            {props.help.clone().unwrap_or_default()}
        </label>
    }
}
