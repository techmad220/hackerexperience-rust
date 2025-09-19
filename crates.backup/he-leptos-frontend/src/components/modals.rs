use leptos::*;

#[component]
pub fn TargetInfoModal() -> impl IntoView {
    view! {
        <div id="target-info-modal" style="display: none;">
            "Target Info Modal"
        </div>
    }
}

#[component]
pub fn VulnScanModal() -> impl IntoView {
    view! {
        <div id="vuln-scan-modal" style="display: none;">
            "Vuln Scan Modal"
        </div>
    }
}

#[component]
pub fn HackProgressModal() -> impl IntoView {
    view! {
        <div id="hack-progress-modal" style="display: none;">
            "Hack Progress Modal"
        </div>
    }
}

#[component]
pub fn BounceRouteModal() -> impl IntoView {
    view! {
        <div id="bounce-route-modal" style="display: none;">
            "Bounce Route Modal"
        </div>
    }
}
