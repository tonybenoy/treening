use crate::models::{Category, Exercise};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub exercises: Vec<Exercise>,
    pub on_select: Callback<Exercise>,
    #[prop_or_default]
    pub on_add: Option<Callback<Exercise>>,
    #[prop_or_default]
    pub show_add_button: bool,
}

/// Simple fuzzy match: checks if all characters of the query appear in order
/// in the target string, allowing gaps. Returns true if the query fuzzy-matches.
fn fuzzy_match(target: &str, query: &str) -> bool {
    let mut target_chars = target.chars();
    for qc in query.chars() {
        loop {
            match target_chars.next() {
                Some(tc) if tc == qc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// Score a fuzzy match — lower is better. Returns None if no match.
/// Prefers: exact substring > prefix > fuzzy with fewer gaps.
fn fuzzy_score(target: &str, query: &str) -> Option<u32> {
    let t = target.to_lowercase();
    let q = query.to_lowercase();

    // Exact substring match
    if t.contains(&q) {
        if t.starts_with(&q) {
            return Some(0); // prefix match — best
        }
        return Some(1); // substring match
    }

    // Fuzzy: chars appear in order with gaps
    if fuzzy_match(&t, &q) {
        // Count total gap size
        let mut gap = 0u32;
        let mut t_iter = t.chars().enumerate();
        for qc in q.chars() {
            loop {
                match t_iter.next() {
                    Some((_, tc)) if tc == qc => break,
                    Some(_) => gap += 1,
                    None => return None,
                }
            }
        }
        return Some(10 + gap);
    }

    // Typo tolerance: check if edit distance on any word is <= 2
    let query_words: Vec<&str> = q.split_whitespace().collect();
    let target_words: Vec<&str> = t.split_whitespace().collect();
    let mut all_matched = !query_words.is_empty();
    let mut total_dist = 0u32;
    for qw in &query_words {
        if let Some(best) = target_words.iter().map(|tw| edit_distance(qw, tw)).min() {
            if best <= 2 {
                total_dist += best;
            } else {
                all_matched = false;
                break;
            }
        } else {
            all_matched = false;
            break;
        }
    }
    if all_matched {
        return Some(100 + total_dist);
    }

    None
}

/// Simple edit distance (Levenshtein) for short strings.
fn edit_distance(a: &str, b: &str) -> u32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = b.len();
    let mut prev = (0..=n as u32).collect::<Vec<_>>();
    let mut curr = vec![0u32; n + 1];
    for (i, ac) in a.iter().enumerate() {
        curr[0] = (i + 1) as u32;
        for (j, bc) in b.iter().enumerate() {
            let cost = if *ac == *bc { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1)
                .min(curr[j] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Score an exercise against a search query. Returns the best (lowest) score
/// across name, muscle groups, equipment, and category.
fn exercise_score(e: &Exercise, query: &str) -> Option<u32> {
    let mut best: Option<u32> = None;
    let mut consider = |s: &str| {
        if let Some(score) = fuzzy_score(s, query) {
            best = Some(best.map_or(score, |b: u32| b.min(score)));
        }
    };
    consider(&e.name);
    for m in &e.muscle_groups {
        consider(m);
    }
    consider(&e.equipment.to_string());
    consider(&e.category.to_string());
    best
}

#[function_component(ExerciseList)]
pub fn exercise_list(props: &Props) -> Html {
    let search = use_state(String::new);
    let category_filter = use_state(|| None::<Category>);

    let mut scored: Vec<(&Exercise, u32)> = props
        .exercises
        .iter()
        .filter_map(|e| {
            let cat_match = match &*category_filter {
                Some(cat) => e.category == *cat,
                None => true,
            };
            if !cat_match {
                return None;
            }
            if search.is_empty() {
                return Some((e, 0));
            }
            exercise_score(e, &search).map(|s| (e, s))
        })
        .collect();
    scored.sort_by_key(|(_, s)| *s);
    let filtered: Vec<&Exercise> = scored.into_iter().map(|(e, _)| e).collect();

    let on_search = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search.set(input.value());
        })
    };

    let categories = Category::all();

    html! {
        <div>
            <div class="px-4 pt-2 pb-2">
                <input
                    type="text"
                    placeholder="Search exercises..."
                    class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors shadow-sm"
                    oninput={on_search}
                />
            </div>
            <div class="px-4 pb-2 flex gap-2 overflow-x-auto scrollbar-hide">
                {
                    {
                        let cf = category_filter.clone();
                        html! {
                            <button
                                class={if cf.is_none() {
                                    "px-3 py-1 rounded-full text-sm bg-blue-600 text-white font-medium shadow-sm transition-colors"
                                } else {
                                    "px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-transparent hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                                }}
                                onclick={let cf = cf.clone(); Callback::from(move |_| cf.set(None))}
                            >{"All"}</button>
                        }
                    }
                }
                { for categories.iter().map(|cat| {
                    let cf = category_filter.clone();
                    let cat_clone = cat.clone();
                    let active = *cf == Some(cat.clone());
                    let label = cat.to_string();
                    html! {
                        <button
                            class={if active {
                                "px-3 py-1 rounded-full text-sm bg-blue-600 text-white whitespace-nowrap font-medium shadow-sm transition-colors"
                            } else {
                                "px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-transparent whitespace-nowrap hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            }}
                            onclick={Callback::from(move |_| cf.set(Some(cat_clone.clone())))}
                        >{label}</button>
                    }
                })}
            </div>
            <div class="px-4 space-y-2 pb-4">
                { for filtered.iter().enumerate().map(|(i, exercise)| {
                    let ex = (*exercise).clone();
                    let on_select = props.on_select.clone();
                    let on_add = props.on_add.clone();
                    let show_add = props.show_add_button;
                    let ex2 = ex.clone();
                    let ex3 = ex.clone();
                    let delay = format!("animation-delay: {}ms", i.min(10) * 30);
                    html! {
                        <div class="bg-gray-100 dark:bg-gray-800 rounded-lg p-3 flex justify-between items-center border border-gray-200 dark:border-transparent transition-colors shadow-sm list-item-enter" style={delay}>
                            <div class="flex-1 cursor-pointer flex items-center gap-3" onclick={Callback::from(move |_| on_select.emit(ex2.clone()))}>
                                { if let Some(ref img) = ex.image {
                                    html! {
                                        <img
                                            src={img.clone()}
                                            alt={ex.name.clone()}
                                            class="w-10 h-10 rounded bg-white dark:bg-gray-700 border border-gray-200 dark:border-transparent p-0.5 flex-shrink-0 transition-colors"
                                        />
                                    }
                                } else {
                                    html! {
                                        <div class="w-10 h-10 rounded bg-white dark:bg-gray-700 border border-gray-200 dark:border-transparent flex items-center justify-center flex-shrink-0 text-gray-400 dark:text-gray-500 text-xs transition-colors">{"?"}</div>
                                    }
                                }}
                                <div>
                                    <div class="font-medium text-gray-900 dark:text-gray-100">{&ex.name}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">
                                        {ex.category.to_string()}{" · "}{ex.equipment.to_string()}
                                    </div>
                                </div>
                            </div>
                            { if show_add {
                                html! {
                                    <button
                                        class="ml-2 px-3 py-1 bg-blue-600 text-white rounded text-sm font-bold hover:bg-blue-700 shadow-sm transition-colors"
                                        onclick={
                                            let on_add = on_add.clone();
                                            Callback::from(move |_| {
                                                if let Some(ref cb) = on_add {
                                                    cb.emit(ex3.clone());
                                                }
                                            })
                                        }
                                    >{"+ Add"}</button>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                })}
                { if filtered.is_empty() {
                    html! { <p class="text-gray-500 dark:text-gray-400 text-center py-12 bg-gray-50 dark:bg-gray-800/20 rounded-2xl border border-dashed border-gray-200 dark:border-gray-700 transition-colors">{"No exercises found"}</p> }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
