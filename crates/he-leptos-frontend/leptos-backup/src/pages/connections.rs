use leptos::*;
use crate::state::*;
use crate::components::*;

#[component]
pub fn ConnectionsPage() -> impl IntoView {
    let game_state = use_context::<RwSignal<GameState>>().unwrap();
    
    view! {
        <div class="content-header">
            <h1>"Connections"</h1>
        </div>
        
        <div id="content">
            <WidgetBox title="Active Connections".to_string()>
                <p>"Connection management functionality will be implemented here."</p>
            </WidgetBox>
        </div>
    }
}