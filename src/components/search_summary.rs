use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchSummaryProps {
    pub search_image: String,
    pub max_mse: f64,
    pub max_results: u16,
    pub on_new_search: Callback<()>,
}

#[function_component(SearchSummary)]
pub fn search_summary(props: &SearchSummaryProps) -> Html {
    html! {
        <div class="search-summary">
            <h2>{"Search summary"}</h2>
            <div class="search-info">
                <div class="search-image-preview">
                    <h3>{"Searched subimage"}</h3>
                    <img
                        src={props.search_image.clone()}
                        alt="Subimage that was searched"
                    />
                </div>
                <div class="settings-summary">
                    <h3>{"Search Settings"}</h3>
                    <span class="setting">{"Maximum difference: "}<strong>{format!("{:.1}%", props.max_mse * 100.0)}</strong></span>
                    <span class="setting">{"Maximum results: "}<strong>{props.max_results}</strong></span>
                </div>
                <button class="edit-button" onclick={props.on_new_search.reform(|_| ())}>{"New Search"}</button>
            </div>
        </div>
    }
}
