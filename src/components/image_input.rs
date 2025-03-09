use wasm_bindgen::JsCast;
use web_sys::{DragEvent, Event, HtmlElement};
use web_sys::{FileList, HtmlInputElement};
use yew::prelude::*;

fn prevent_default(event: &Event) {
    event.prevent_default();
    event.stop_propagation();
}

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

fn on_drop(props: &ImageInputProps, label_ref: &NodeRef) -> Callback<DragEvent> {
    if props.disabled {
        Callback::from(|event: DragEvent| {
            prevent_default(&event);
        })
    } else {
        let on_upload = props.on_upload.clone();
        let label_ref = label_ref.clone();
        Callback::from(move |event: DragEvent| {
            prevent_default(&event);
            node_ref_toggle_class(&label_ref, "dragover", false);
            if let Some(files) = event.data_transfer().and_then(|x| x.files()) {
                on_upload.emit(files.clone());
            }
        })
    }
}

fn on_drag_leave(label_ref: &NodeRef) -> Callback<DragEvent> {
    let label_ref = label_ref.clone();
    Callback::from(move |event: DragEvent| {
        prevent_default(&event);
        node_ref_toggle_class(&label_ref, "dragover", false);
    })
}

fn on_drag_start(props: &ImageInputProps, label_ref: &NodeRef) -> Callback<DragEvent> {
    let disabled = props.disabled;
    let label_ref = label_ref.clone();
    Callback::from(move |event: DragEvent| {
        prevent_default(&event);
        if !disabled {
            node_ref_toggle_class(&label_ref, "dragover", true);
        }
    })
}

fn node_ref_toggle_class(node_ref: &NodeRef, class: &str, add: bool) {
    if let Some(element) = node_ref.cast::<HtmlElement>() {
        element.class_list().toggle_with_force(class, add).unwrap();
    } else {
        panic!("Failed to cast element to HtmlElement: {:?}", node_ref);
    }
}

#[function_component(ImageInput)]
pub fn image_input(props: &ImageInputProps) -> Html {
    let label_ref = use_node_ref();
    html! {
        <label
            ref={&label_ref}
            class={if props.disabled {"image-input disabled"} else {"image-input"}}
            id={format!("{}-container", props.input_id)}
            ondrop={on_drop(props, &label_ref)}
            ondragenter={on_drag_start(props, &label_ref)}
            ondragover={on_drag_start(props, &label_ref)}
            ondragleave={on_drag_leave(&label_ref)}
        >
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
