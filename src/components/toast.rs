use gloo::timers::callback::Timeout;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ToastProps {
    pub message: AttrValue,
    pub visible: bool,
    pub on_dismiss: Callback<()>,
}

#[function_component(Toast)]
pub fn toast(props: &ToastProps) -> Html {
    let on_dismiss = props.on_dismiss.clone();
    let visible = props.visible;

    {
        let on_dismiss = on_dismiss.clone();
        use_effect_with(visible, move |visible| {
            let timeout = if *visible {
                Some(Timeout::new(3000, move || {
                    on_dismiss.emit(());
                }))
            } else {
                None
            };
            move || drop(timeout)
        });
    }

    if !props.visible {
        return html! {};
    }

    html! {
        <div
            class="fixed top-4 left-4 right-4 z-[60] flex justify-center pointer-events-none"
            style="animation: modalContentIn 200ms ease-out;"
        >
            <div class="bg-yellow-500 text-yellow-950 px-5 py-3 rounded-xl shadow-lg text-center font-bold text-sm pointer-events-auto neu-btn flex items-center gap-2">
                <span class="text-lg">{"🏆"}</span>
                <span>{&props.message}</span>
            </div>
        </div>
    }
}
