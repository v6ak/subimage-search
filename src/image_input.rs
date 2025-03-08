use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ImageInputProps {
    pub label: String,
    pub input_id: String,
    pub preview_id: String,
    pub onchange: Callback<Event>,
    pub image: Option<String>,
    pub help: Option<Html>,
}

#[function_component(ImageInput)]
pub fn image_input(props: &ImageInputProps) -> Html {
    html! {
        <label class="image-input" id={format!("{}-container", props.input_id)}>
            <h3>{&props.label}</h3>
            <input
                type="file"
                id={props.input_id.clone()}
                accept="image/*"
                onchange={props.onchange.clone()}
            />
            {
                if props.image.is_none() {
                    html! {
                        <p>{"Click here to select an image"}</p>
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
