use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlElement};

pub fn prevent_default(event: &Event) {
    event.prevent_default();
    event.stop_propagation();
}

pub fn setup_drag_handlers(element: &HtmlElement) {
    let el = element.clone();
    let dragenter = Closure::wrap(Box::new(move |e: Event| {
        prevent_default(&e);
        el.set_class_name("image-input dragover");
    }) as Box<dyn FnMut(_)>);
    element
        .add_event_listener_with_callback("dragenter", dragenter.as_ref().unchecked_ref())
        .unwrap();
    dragenter.forget();

    let el = element.clone();
    let dragover = Closure::wrap(Box::new(move |e: Event| {
        prevent_default(&e);
        el.set_class_name("image-input dragover");
    }) as Box<dyn FnMut(_)>);
    element
        .add_event_listener_with_callback("dragover", dragover.as_ref().unchecked_ref())
        .unwrap();
    dragover.forget();

    let el = element.clone();
    let dragleave = Closure::wrap(Box::new(move |e: Event| {
        prevent_default(&e);
        el.set_class_name("image-input");
    }) as Box<dyn FnMut(_)>);
    element
        .add_event_listener_with_callback("dragleave", dragleave.as_ref().unchecked_ref())
        .unwrap();
    dragleave.forget();
}
