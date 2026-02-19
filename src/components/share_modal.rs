use yew::prelude::*;
use crate::sharing::{self, ShareableData};

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub shareable: ShareableData,
    pub formatted_text: String,
    pub on_close: Callback<()>,
}

impl PartialEq for ShareableData {
    fn eq(&self, _other: &Self) -> bool {
        false // always re-render
    }
}

#[function_component(ShareModal)]
pub fn share_modal(props: &Props) -> Html {
    let copied_link = use_state(|| false);
    let copied_text = use_state(|| false);

    let title = match &props.shareable {
        ShareableData::Workout { .. } => "Share Workout",
        ShareableData::Routine { .. } => "Share Routine",
        ShareableData::Exercise { .. } => "Share Exercise",
    };

    let share_url = sharing::build_share_url(&props.shareable);

    let qr_html = if let Ok(ref url) = share_url {
        let result: Result<String, _> = qrcode_generator::to_svg_to_string(
            url,
            qrcode_generator::QrCodeEcc::Low,
            400,
            None::<&str>,
        );
        match result {
            Ok(svg) => {
                let base64 = gloo::utils::window().btoa(&svg).unwrap_or_default();
                html! {
                    <div class="flex justify-center mb-4">
                        <img
                            src={format!("data:image/svg+xml;base64,{}", base64)}
                            alt="QR Code"
                            class="w-48 h-48 bg-white rounded-lg p-2"
                        />
                    </div>
                }
            }
            Err(_) => html! {},
        }
    } else {
        html! {}
    };

    let copy_link_btn = if let Ok(ref url) = share_url {
        let url = url.clone();
        let copied = copied_link.clone();
        let onclick = Callback::from(move |_| {
            let nav = gloo::utils::window().navigator();
            let clipboard = nav.clipboard();
            let _ = clipboard.write_text(&url);
            copied.set(true);
            let copied = copied.clone();
            gloo::timers::callback::Timeout::new(2000, move || {
                copied.set(false);
            })
            .forget();
        });
        let label = if *copied_link { "Copied!" } else { "Copy Link" };
        html! {
            <button
                class="w-full py-2.5 bg-blue-600 text-white rounded-lg text-sm font-bold hover:bg-blue-700 shadow-sm transition-colors"
                onclick={onclick}
            >{label}</button>
        }
    } else {
        html! {
            <div class="text-sm text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg px-3 py-2 text-center">
                {"Data too large for a shareable URL. Use \"Copy as Text\" instead."}
            </div>
        }
    };

    let formatted = props.formatted_text.clone();
    let copied_t = copied_text.clone();
    let copy_text_click = Callback::from(move |_| {
        let nav = gloo::utils::window().navigator();
        let clipboard = nav.clipboard();
        let _ = clipboard.write_text(&formatted);
        copied_t.set(true);
        let copied_t = copied_t.clone();
        gloo::timers::callback::Timeout::new(2000, move || {
            copied_t.set(false);
        })
        .forget();
    });
    let text_label = if *copied_text { "Copied!" } else { "Copy as Text" };

    let on_close = props.on_close.clone();
    let on_close2 = props.on_close.clone();

    html! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm px-4 modal-overlay-enter"
            onclick={Callback::from(move |_| on_close.emit(()))}
        >
            <div
                class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl w-full max-w-sm p-6 relative modal-content-enter"
                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
            >
                <button
                    class="absolute top-3 right-3 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl leading-none"
                    onclick={Callback::from(move |_| on_close2.emit(()))}
                >{"\u{2715}"}</button>

                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100 mb-4">{title}</h2>

                {qr_html}

                <div class="space-y-2">
                    {copy_link_btn}
                    <button
                        class="w-full py-2.5 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg text-sm font-bold hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
                        onclick={copy_text_click}
                    >{text_label}</button>
                </div>
            </div>
        </div>
    }
}
