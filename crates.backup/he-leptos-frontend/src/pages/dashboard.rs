use leptos::*;
use crate::api;

#[component]
pub fn DashboardPage() -> impl IntoView {
    // Fetch real data from API
    let dashboard_resource = create_resource(
        || (),
        |_| async move {
            api::get_dashboard().await
        }
    );

    let hardware_resource = create_resource(
        || (),
        |_| async move {
            api::get_hardware().await
        }
    );

    let processes_resource = create_resource(
        || (),
        |_| async move {
            api::get_processes().await
        }
    );

    view! {
        <div class="row-fluid">
            <div class="span5">
                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"Control Panel"</h5>
                    </div>
                    <div class="widget-content">
                        <Suspense fallback=move || view! { <p>"Loading hardware..."</p> }>
                            {move || hardware_resource.get().map(|result| match result {
                                Ok(hw) => view! {
                                    <table class="table table-bordered table-striped table-cozy">
                                        <tbody>
                                            <tr>
                                                <td>"Processor"</td>
                                                <td><span class="item">{format!("{:.1} GHz", hw.cpu_speed)}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Hard Drive"</td>
                                                <td><span class="item">{format!("{} GB ({} GB used)", hw.hdd_size / 1000, hw.hdd_used / 1000)}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Memory"</td>
                                                <td><span class="item">{format!("{} MB", hw.ram_size)}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Internet"</td>
                                                <td><span class="item">{format!("{:.1} Mbps", hw.net_speed)}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"External HD"</td>
                                                <td><span class="item">"None"</span></td>
                                            </tr>
                                        </tbody>
                                    </table>
                                },
                                Err(e) => view! {
                                    <div class="alert alert-error">{e}</div>
                                }
                            })}
                        </Suspense>
                    </div>
                </div>
            </div>

            <div class="span7">
                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"General Info"</h5>
                    </div>
                    <div class="widget-content">
                        <Suspense fallback=move || view! { <p>"Loading status..."</p> }>
                            {move || dashboard_resource.get().map(|result| match result {
                                Ok(data) => view! {
                                    <table class="table table-bordered table-striped table-cozy">
                                        <tbody>
                                            <tr>
                                                <td>"Reputation"</td>
                                                <td><span class="green">{format!("{}", data.status.reputation)}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Level"</td>
                                                <td><span class="item">{data.status.level}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Experience"</td>
                                                <td><span class="item">{data.status.experience}" XP"</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Bank Balance"</td>
                                                <td><span class="item">"$"{data.bank_balance}</span></td>
                                            </tr>
                                            <tr>
                                                <td>"Messages"</td>
                                                <td><span class="item">{data.unread_messages}" unread"</span></td>
                                            </tr>
                                        </tbody>
                                    </table>
                                },
                                Err(e) => view! {
                                    <div class="alert alert-error">{e}</div>
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                <div class="widget-box">
                    <div class="widget-title">
                        <h5>"System info"</h5>
                    </div>
                    <div class="widget-content">
                        <Suspense fallback=move || view! { <p>"Loading processes..."</p> }>
                            {move || {
                                let dashboard = dashboard_resource.get();
                                let processes = processes_resource.get();

                                view! {
                                    <table class="table table-bordered table-striped table-cozy">
                                        <tbody>
                                            <tr>
                                                <td>"Running tasks"</td>
                                                <td><span class="item">
                                                    {move || processes.as_ref().map(|p| p.as_ref().map(|procs| procs.len()).unwrap_or(0)).unwrap_or(0)}
                                                </span></td>
                                            </tr>
                                            <tr>
                                                <td>"Hardware Load"</td>
                                                <td><span class="item">
                                                    {move || dashboard.as_ref().and_then(|d| d.as_ref().ok()).map(|data| format!("{:.1}%", data.hardware_load)).unwrap_or_else(|| "0%".to_string())}
                                                </span></td>
                                            </tr>
                                            <tr>
                                                <td>"Status"</td>
                                                <td><span class="green">"Online"</span></td>
                                            </tr>
                                        </tbody>
                                    </table>
                                }
                            }}
                        </Suspense>
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