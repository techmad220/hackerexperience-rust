use leptos::*;

#[component]
pub fn WidgetBox(
    title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="widget-box">
            <div class="widget-title">
                <h5>{title}</h5>
            </div>
            <div class="widget-content">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn Terminal() -> impl IntoView {
    view! {
        <div class="terminal">
            <div>"root@localhost:~# Welcome to Hacker Experience"</div>
            <div>"System initialized successfully."</div>
            <div>"Type 'help' for available commands."</div>
        </div>
    }
}

#[component]
pub fn QuickActions() -> impl IntoView {
    view! {
        <div>
            <p><a href="/processes" class="btn btn-primary btn-block">"View Processes"</a></p>
            <p><a href="/software" class="btn btn-primary btn-block">"Software Manager"</a></p>
            <p><a href="/internet" class="btn btn-primary btn-block">"Internet Browser"</a></p>
            <p><a href="/missions" class="btn btn-primary btn-block">"Mission Center"</a></p>
        </div>
    }
}

#[component]
pub fn SystemStatus() -> impl IntoView {
    view! {
        <table class="table">
            <tr>
                <td>"CPU Usage:"</td>
                <td>
                    <div class="progress">
                        <div class="progress-bar" style="width: 25%">"25%"</div>
                    </div>
                </td>
            </tr>
            <tr>
                <td>"Memory:"</td>
                <td>
                    <div class="progress">
                        <div class="progress-bar" style="width: 45%">"45%"</div>
                    </div>
                </td>
            </tr>
            <tr>
                <td>"Disk:"</td>
                <td>
                    <div class="progress">
                        <div class="progress-bar" style="width: 12%">"12%"</div>
                    </div>
                </td>
            </tr>
            <tr>
                <td>"Network:"</td>
                <td><span class="process-running">"Connected"</span></td>
            </tr>
        </table>
    }
}