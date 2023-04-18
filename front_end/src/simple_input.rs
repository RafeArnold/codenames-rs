use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SimpleInputProps {
    pub value: String,
    pub set_value: Callback<String>,
    pub label_name: String,
}

#[function_component]
pub fn SimpleInput(props: &SimpleInputProps) -> Html {
    let SimpleInputProps {
        value,
        set_value,
        label_name,
    } = props;

    let set_value_clone = set_value.clone();
    let oninput = move |event: InputEvent| {
        set_value_clone.emit(event.target_unchecked_into::<HtmlInputElement>().value())
    };

    let player_name_clone = value.clone();

    html! {
        <>
            <label>{label_name}
                <input {oninput} value={player_name_clone}/>
            </label>
        </>
    }
}
