use crate::models::Exercise;
use crate::storage;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub exercise: Exercise,
    pub on_back: Callback<()>,
    #[prop_or_default]
    pub on_share: Option<Callback<Exercise>>,
}

#[function_component(ExerciseDetail)]
pub fn exercise_detail(props: &Props) -> Html {
    let ex = &props.exercise;
    let on_back = props.on_back.clone();
    let show_video = use_state(|| false);

    let video_demos_enabled = storage::load_user_config().video_demos_enabled;

    let on_toggle_video = {
        let show_video = show_video.clone();
        Callback::from(move |_| show_video.set(!*show_video))
    };

    let active_video_id = if video_demos_enabled {
        ex.video_id.clone()
    } else {
        None
    };

    html! {
        <div class="px-4 py-4 transition-colors duration-200">
            <div class="flex justify-between items-center mb-4">
                <button
                    class="text-blue-600 dark:text-blue-400 flex items-center gap-1 font-medium hover:underline"
                    onclick={Callback::from(move |_| on_back.emit(()))}
                >
                    {"\u{2190} Back"}
                </button>
                { if ex.is_custom {
                    if let Some(ref on_share) = props.on_share {
                        let on_share = on_share.clone();
                        let ex_c = ex.clone();
                        html! {
                            <button
                                class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-bold hover:bg-green-700 neu-btn transition-colors"
                                onclick={Callback::from(move |_| on_share.emit(ex_c.clone()))}
                            >{"Share"}</button>
                        }
                    } else { html! {} }
                } else { html! {} }}
            </div>
            <h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-gray-100">{&ex.name}</h2>
            <div class="flex flex-wrap gap-2 mb-6">
                <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-transparent rounded text-xs font-bold uppercase tracking-wider">{ex.category.to_string()}</span>
                <span class="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded text-xs font-bold uppercase tracking-wider neu-chip">{ex.equipment.to_string()}</span>
                { if ex.is_custom {
                    html! { <span class="px-2 py-1 bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-transparent rounded text-xs font-bold uppercase tracking-wider">{"Custom"}</span> }
                } else {
                    html! {}
                }}
            </div>
            { if let Some(ref img) = ex.image {
                html! {
                    <div class="mb-6 flex justify-center">
                        <img
                            src={img.clone()}
                            alt={ex.name.clone()}
                            class="w-full max-w-sm rounded-xl bg-gray-100 dark:bg-gray-800 p-2 neu-flat transition-colors"
                        />
                    </div>
                }
            } else {
                html! {}
            }}
            { if let Some(ref vid) = active_video_id {
                let vid_clone = vid.clone();
                html! {
                    <div class="mb-6">
                        { if *show_video {
                            html! {
                                <div>
                                    <div class="relative w-full rounded-xl overflow-hidden neu-flat" style="padding-bottom: 56.25%;">
                                        <iframe
                                            class="absolute top-0 left-0 w-full h-full"
                                            src={format!("https://www.youtube-nocookie.com/embed/{}", vid_clone)}
                                            title="Exercise demo"
                                            frameborder="0"
                                            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                                            allowfullscreen=true
                                        ></iframe>
                                    </div>
                                    <div class="flex items-center justify-between mt-2">
                                        <a
                                            href={format!("https://www.youtube.com/watch?v={}", vid_clone)}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                        >{"Open in YouTube \u{2197}"}</a>
                                        <button
                                            class="text-sm text-gray-500 dark:text-gray-400 hover:underline"
                                            onclick={on_toggle_video}
                                        >{"Hide video"}</button>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <button
                                    class="w-full py-2.5 bg-red-600 hover:bg-red-700 text-white rounded-lg font-bold neu-btn transition-colors flex items-center justify-center gap-2"
                                    onclick={on_toggle_video}
                                >
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                                    </svg>
                                    {"Watch Demo"}
                                </button>
                            }
                        }}
                    </div>
                }
            } else {
                html! {}
            }}
            <div class="mb-6">
                <h3 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">{"Muscles Worked"}</h3>
                <div class="flex flex-wrap gap-2">
                    { for ex.muscle_groups.iter().map(|m| html! {
                        <span class="px-3 py-1 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm text-gray-700 dark:text-gray-300 neu-chip transition-colors">{m}</span>
                    })}
                </div>
            </div>
            <div class="bg-gray-100 dark:bg-gray-800/50 rounded-2xl p-4 neu-flat transition-colors">
                <h3 class="text-lg font-bold mb-2 text-gray-900 dark:text-gray-100">{"How to Perform"}</h3>
                <p class="text-gray-600 dark:text-gray-400 leading-relaxed text-sm">{&ex.description}</p>
            </div>
        </div>
    }
}
