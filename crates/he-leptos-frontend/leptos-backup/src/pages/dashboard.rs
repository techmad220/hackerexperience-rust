use leptos::*;
use crate::api::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="row-fluid">
            <div class="span5">
                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"Control Panel"</h5>
                    </div>
                    <div class="widget-content">
                        <table class="table table-bordered table-striped table-cozy">
                            <tbody>
                                <tr>
                                    <td>"Processor"</td>
                                    <td><span class="item">"AMD K6-2 500MHz"</span></td>
                                </tr>
                                <tr>
                                    <td>"Hard Drive"</td>
                                    <td><span class="item">"10 GB Maxtor"</span></td>
                                </tr>
                                <tr>
                                    <td>"Memory"</td>
                                    <td><span class="item">"128 MB"</span></td>
                                </tr>
                                <tr>
                                    <td>"Internet"</td>
                                    <td><span class="item">"Modem 56K"</span></td>
                                </tr>
                                <tr>
                                    <td>"External HD"</td>
                                    <td><span class="item">"None"</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="span7">
                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"General Info"</h5>
                    </div>
                    <div class="widget-content">
                        <table class="table table-bordered table-striped table-cozy">
                            <tbody>
                                <tr>
                                    <td>"Reputation"</td>
                                    <td><span class="green">"Neutral (0)"</span></td>
                                </tr>
                                <tr>
                                    <td>"Running tasks"</td>
                                    <td><span class="item">"0"</span></td>
                                </tr>
                                <tr>
                                    <td>"Connections"</td>
                                    <td><span class="item">"0"</span></td>
                                </tr>
                                <tr>
                                    <td>"Mission"</td>
                                    <td><span class="item">"None"</span></td>
                                </tr>
                                <tr>
                                    <td>"Clan"</td>
                                    <td><span class="item">"None"</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>

                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"System info"</h5>
                    </div>
                    <div class="widget-content">
                        <table class="table table-bordered table-striped table-cozy">
                            <tbody>
                                <tr>
                                    <td>"Uptime"</td>
                                    <td><span class="item">"42 minutes"</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let game_data = create_api_resource();

    view! {
        <Suspense fallback=move || view! { <div>"Loading game data..."</div> }>
            {move || {
                game_data.get().map(|data| match data {
                    Ok(game_data) => view! {
                        <div class="row-fluid">
                            <div class="span5">
                                <div class="widget-box">
                                    <div class="widget-title">
                                        <h5>"Control Panel"</h5>
                                    </div>
                                    <div class="widget-content">
                                        <table class="table table-bordered table-striped table-cozy">
                                            <tbody>
                                                <tr>
                                                    <td>"Processor"</td>
                                                    <td><span class="item">{&game_data.hardware.processor}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Hard Drive"</td>
                                                    <td><span class="item">{&game_data.hardware.hard_drive}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Memory"</td>
                                                    <td><span class="item">{&game_data.hardware.memory}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Internet"</td>
                                                    <td><span class="item">{&game_data.hardware.internet}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"External HD"</td>
                                                    <td><span class="item">{game_data.hardware.external_hd.clone().unwrap_or_else(|| "None".to_string())}</span></td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>

                            <div class="span7">
                                <div class="widget-box">
                                    <div class="widget-title">
                                        <h5>"General Info"</h5>
                                    </div>
                                    <div class="widget-content">
                                        <table class="table table-bordered table-striped table-cozy">
                                            <tbody>
                                                <tr>
                                                    <td>"Reputation"</td>
                                                    <td><span class="green">{format!("Neutral ({})", game_data.user.reputation)}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Running tasks"</td>
                                                    <td><span class="item">{game_data.system_info.running_tasks}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Connections"</td>
                                                    <td><span class="item">{game_data.system_info.connections}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Mission"</td>
                                                    <td><span class="item">{game_data.system_info.mission.clone().unwrap_or_else(|| "None".to_string())}</span></td>
                                                </tr>
                                                <tr>
                                                    <td>"Clan"</td>
                                                    <td><span class="item">{game_data.system_info.clan.clone().unwrap_or_else(|| "None".to_string())}</span></td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div>
                                </div>

                                <div class="widget-box">
                                    <div class="widget-title">
                                        <h5>"System info"</h5>
                                    </div>
                                    <div class="widget-content">
                                        <table class="table table-bordered table-striped table-cozy">
                                            <tbody>
                                                <tr>
                                                    <td>"Uptime"</td>
                                                    <td><span class="item">{&game_data.system_info.uptime}</span></td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view(),
                    Err(err) => view! {
                        <div class="row-fluid">
                            <div class="span12">
                                <div class="widget-box">
                                    <div class="widget-title">
                                        <h5>"Error"</h5>
                                    </div>
                                    <div class="widget-content">
                                        <p style="color: #ff0000;">{format!("Failed to load game data: {}", err)}</p>
                                        <p>"Using offline mode with default values."</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                })
            }}
        </Suspense>
    }
}