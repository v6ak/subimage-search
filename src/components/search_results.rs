use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchErrorProps {
    pub message: AttrValue,
}

#[function_component(SearchError)]
pub fn search_error(props: &SearchErrorProps) -> Html {
    html! {
        <div class="result-container">
            <h2>{"Error"}</h2>
            <div class="error-message">
                <h3>{"An error occurred during processing"}</h3>
                <p>{&props.message}</p>
            </div>
        </div>
    }
}
