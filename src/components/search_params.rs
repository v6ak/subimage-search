use crate::image_input::ImageInput;
use web_sys::{FileList, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchParamsProps {
    pub main_image: Option<String>,
    pub search_image: Option<String>,
    pub max_mse: f64,
    pub max_results: u16,
    pub disabled: bool,
    pub on_max_mse_change: Callback<f64>,
    pub on_max_results_change: Callback<u16>,
    pub on_main_image_upload: Callback<FileList>,
    pub on_search_image_upload: Callback<FileList>,
}

#[function_component(SearchParams)]
pub fn search_params(props: &SearchParamsProps) -> Html {
    let on_max_mse_change = props.on_max_mse_change.clone();
    let on_max_results_change = props.on_max_results_change.clone();

    let handle_mse_change = Callback::from(move |e: InputEvent| {
        let value = e
            .target_dyn_into::<HtmlInputElement>()
            .unwrap()
            .value()
            .parse::<f64>()
            .unwrap();
        on_max_mse_change.emit(value);
    });

    let handle_results_change = Callback::from(move |e: InputEvent| {
        let value = e
            .target_dyn_into::<HtmlInputElement>()
            .unwrap()
            .value()
            .parse::<u16>()
            .unwrap();
        on_max_results_change.emit(value);
    });

    html! {
        <>
            <h2>{"Images"}</h2>
            <div class="image-inputs">
                <ImageInput
                    label="Main Image"
                    input_id="mainImageInput"
                    preview_id="mainImagePreview"
                    on_upload={props.on_main_image_upload.clone()}
                    image={props.main_image.clone()}
                    help={None}
                    disabled={props.disabled}
                />
                <ImageInput
                    label="Image to search"
                    input_id="searchImageInput"
                    preview_id="searchImagePreview"
                    on_upload={props.on_search_image_upload.clone()}
                    image={props.search_image.clone()}
                    help={Some(image_search_help())}
                    disabled={props.disabled}
                />
            </div>

            <h2>{"Settings"}</h2>
            <div class="settings">
                <label class="settings-item">
                    <h3>{"Maximum difference (%)"}</h3>
                    <input
                        type="number"
                        id="maxMseInput"
                        value={(props.max_mse * 100.0).to_string()}
                        oninput={handle_mse_change}
                        disabled={props.disabled}
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
                        value={props.max_results.to_string()}
                        oninput={handle_results_change}
                        disabled={props.disabled}
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
        </>
    }
}

fn image_search_help() -> Html {
    html! {
        <ul class="image-hint">
            <li><strong>{"Orientation"}</strong>{" has to be the same as in main image."}</li>
            <li><strong>{"Scale"}</strong>{" has to be the same as in main image."}</li>
            <li><strong>{"Compression artifacts and blur caused by scaling up"}</strong>{" can be handled by increasing the maximum difference."}</li>
            <li><strong>{"Alpha channel"}</strong>{" cannot be used as a wildcard. It is considered as a color component. If you don't know the consequences, you probably don't want to search an image with significant transparency."}</li>
        </ul>
    }
}
