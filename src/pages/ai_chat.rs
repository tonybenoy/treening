use crate::components::ai_chat::AiChat;
use yew::prelude::*;

#[function_component(AiChatPage)]
pub fn ai_chat_page() -> Html {
    html! {
        <div class="max-w-2xl mx-auto">
            <AiChat />
        </div>
    }
}
