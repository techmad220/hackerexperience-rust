use leptos::*;
use leptos_router::*;
use crate::api::hacking::{
    scan_server, hack_server, server_action, get_internet_view,
    ScanResponse, HackResponse, ServerActionResponse, InternetResponse,
    ServerInfo, KnownServer, BountyInfo
};

#[component]
pub fn InternetPage() -> impl IntoView {
    // State for IP input
    let (target_ip, set_target_ip) = create_signal(String::from("1.2.3.4"));
    let (scan_result, set_scan_result) = create_signal::<Option<ServerInfo>>(None);
    let (hack_status, set_hack_status) = create_signal::<Option<String>>(None);
    let (server_files, set_server_files) = create_signal::<Vec<String>>(Vec::new());
    let (server_money, set_server_money) = create_signal::<i64>(0);
    let (server_logs, set_server_logs) = create_signal::<Vec<String>>(Vec::new());

    // Internet view data
    let (known_servers, set_known_servers) = create_signal::<Vec<KnownServer>>(Vec::new());
    let (bounties, set_bounties) = create_signal::<Vec<BountyInfo>>(Vec::new());
    let (your_ip, set_your_ip) = create_signal(String::from("Unknown"));

    // Loading states
    let (scanning, set_scanning) = create_signal(false);
    let (hacking, set_hacking) = create_signal(false);
    let (has_access, set_has_access) = create_signal(false);

    // Load internet view on mount
    create_effect(move |_| {
        spawn_local(async move {
            match get_internet_view().await {
                Ok(data) => {
                    set_your_ip.set(data.your_ip);
                    set_known_servers.set(data.known_servers);
                    set_bounties.set(data.bounties);
                }
                Err(e) => {
                    logging::log!("Failed to load internet view: {}", e);
                }
            }
        });
    });

    // Scan server action
    let scan_action = create_action(move |ip: &String| {
        let ip = ip.clone();
        async move {
            set_scanning.set(true);
            set_scan_result.set(None);
            set_hack_status.set(None);
            set_has_access.set(false);

            match scan_server(ip).await {
                Ok(response) => {
                    if let Some(info) = response.server_info {
                        set_scan_result.set(Some(info));
                    }
                    set_hack_status.set(Some(response.message));
                }
                Err(e) => {
                    set_hack_status.set(Some(format!("Scan failed: {}", e)));
                }
            }
            set_scanning.set(false);
        }
    });

    // Hack server action
    let hack_action = create_action(move |method: &String| {
        let ip = target_ip.get();
        let method = method.clone();
        async move {
            set_hacking.set(true);
            set_hack_status.set(Some("Initiating hack...".to_string()));

            match hack_server(ip, method).await {
                Ok(response) => {
                    if response.success {
                        if let Some(time) = response.estimated_time {
                            set_hack_status.set(Some(format!(
                                "Hacking in progress... ETA: {} seconds",
                                time
                            )));
                            // Simulate waiting for hack to complete
                            gloo_timers::future::TimeoutFuture::new(
                                (time * 1000) as u32
                            ).await;
                            set_has_access.set(true);
                            set_hack_status.set(Some("Access granted!".to_string()));
                        }
                    } else {
                        set_hack_status.set(Some(response.message));
                    }
                }
                Err(e) => {
                    set_hack_status.set(Some(format!("Hack failed: {}", e)));
                }
            }
            set_hacking.set(false);
        }
    });

    // Server actions (download, transfer, logs)
    let list_files_action = create_action(move |_: &()| {
        let ip = target_ip.get();
        async move {
            match server_action(ip, "download".to_string(), None).await {
                Ok(response) => {
                    if let Some(data) = response.data {
                        if let Ok(files) = serde_json::from_value::<Vec<String>>(data) {
                            set_server_files.set(files);
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to list files: {}", e);
                }
            }
        }
    });

    let check_money_action = create_action(move |_: &()| {
        let ip = target_ip.get();
        async move {
            match server_action(ip, "transfer".to_string(), None).await {
                Ok(response) => {
                    if let Some(data) = response.data {
                        if let Some(money) = data.get("available") {
                            if let Some(amount) = money.as_i64() {
                                set_server_money.set(amount);
                            }
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to check money: {}", e);
                }
            }
        }
    });

    let view_logs_action = create_action(move |_: &()| {
        let ip = target_ip.get();
        async move {
            match server_action(ip, "logs".to_string(), None).await {
                Ok(response) => {
                    if let Some(data) = response.data {
                        if let Ok(logs) = serde_json::from_value::<Vec<String>>(data) {
                            set_server_logs.set(logs);
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to view logs: {}", e);
                }
            }
        }
    });

    let delete_logs_action = create_action(move |_: &()| {
        let ip = target_ip.get();
        async move {
            match server_action(ip, "logs".to_string(), Some("delete".to_string())).await {
                Ok(response) => {
                    set_hack_status.set(Some(response.message));
                    set_server_logs.set(Vec::new());
                }
                Err(e) => {
                    set_hack_status.set(Some(format!("Failed to delete logs: {}", e)));
                }
            }
        }
    });

    let transfer_money_action = create_action(move |amount: &String| {
        let ip = target_ip.get();
        let amount = amount.clone();
        async move {
            match server_action(ip, "transfer".to_string(), Some(amount)).await {
                Ok(response) => {
                    set_hack_status.set(Some(response.message));
                    set_server_money.set(0);
                }
                Err(e) => {
                    set_hack_status.set(Some(format!("Transfer failed: {}", e)));
                }
            }
        }
    });

    view! {
        <div class="internet-page p-4">
            <h1 class="text-3xl font-bold mb-4">"Internet"</h1>

            // Your IP
            <div class="mb-4">
                <span class="text-sm text-gray-400">"Your IP: "</span>
                <span class="font-mono">{move || your_ip.get()}</span>
            </div>

            // IP Address Bar
            <div class="ip-scanner mb-6">
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 px-3 py-2 bg-gray-800 border border-gray-600 rounded"
                        placeholder="Enter IP address (e.g., 1.2.3.4)"
                        on:input=move |ev| {
                            set_target_ip.set(event_target_value(&ev));
                        }
                        prop:value=move || target_ip.get()
                    />
                    <button
                        class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-semibold"
                        on:click=move |_| {
                            scan_action.dispatch(target_ip.get());
                        }
                        disabled=move || scanning.get()
                    >
                        {move || if scanning.get() { "Scanning..." } else { "Scan" }}
                    </button>
                </div>
            </div>

            // Status Messages
            {move || hack_status.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-gray-800 border border-gray-600 rounded">
                    <span class="text-yellow-400">{msg}</span>
                </div>
            })}

            // Scan Result
            {move || scan_result.get().map(|info| view! {
                <div class="server-info mb-6 p-4 bg-gray-800 rounded">
                    <h2 class="text-xl font-bold mb-3">"Server Information"</h2>
                    <div class="grid grid-cols-2 gap-2 text-sm">
                        <div>
                            <span class="text-gray-400">"Hostname: "</span>
                            <span class="font-mono">{info.hostname.clone()}</span>
                        </div>
                        <div>
                            <span class="text-gray-400">"Owner: "</span>
                            <span>{info.owner.clone()}</span>
                        </div>
                        <div>
                            <span class="text-gray-400">"Type: "</span>
                            <span>{info.server_type.clone()}</span>
                        </div>
                        <div>
                            <span class="text-gray-400">"Security: "</span>
                            <span class="text-yellow-400">{info.security_level}</span>
                        </div>
                        <div>
                            <span class="text-gray-400">"Firewall: "</span>
                            <span class="text-orange-400">{info.firewall_level}</span>
                        </div>
                        <div>
                            <span class="text-gray-400">"Status: "</span>
                            <span class={if info.is_online { "text-green-400" } else { "text-red-400" }}>
                                {if info.is_online { "Online" } else { "Offline" }}
                            </span>
                        </div>
                    </div>

                    // Hack Options
                    {move || if !has_access.get() {
                        view! {
                            <div class="mt-4 flex gap-2">
                                <button
                                    class="px-3 py-1 bg-red-600 hover:bg-red-700 rounded"
                                    on:click=move |_| {
                                        hack_action.dispatch("password".to_string());
                                    }
                                    disabled=move || hacking.get()
                                >
                                    "Password Crack"
                                </button>
                                <button
                                    class="px-3 py-1 bg-purple-600 hover:bg-purple-700 rounded"
                                    on:click=move |_| {
                                        hack_action.dispatch("exploit".to_string());
                                    }
                                    disabled=move || hacking.get()
                                >
                                    "Exploit"
                                </button>
                                <button
                                    class="px-3 py-1 bg-orange-600 hover:bg-orange-700 rounded"
                                    on:click=move |_| {
                                        hack_action.dispatch("brute_force".to_string());
                                    }
                                    disabled=move || hacking.get()
                                >
                                    "Brute Force"
                                </button>
                            </div>
                        }
                    } else {
                        view! {
                            <div></div>
                        }
                    }}
                </div>
            })}

            // Server Actions (when hacked)
            {move || if has_access.get() {
                view! {
                    <div class="server-actions mb-6 p-4 bg-gray-800 rounded">
                        <h2 class="text-xl font-bold mb-3 text-green-400">"Server Access Granted"</h2>

                        <div class="grid grid-cols-3 gap-4">
                            // Files Section
                            <div class="border border-gray-600 p-3 rounded">
                                <h3 class="font-semibold mb-2">"Files"</h3>
                                <button
                                    class="px-2 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm mb-2"
                                    on:click=move |_| { list_files_action.dispatch(()); }
                                >
                                    "List Files"
                                </button>
                                <div class="text-sm space-y-1">
                                    {move || server_files.get().iter().map(|file| {
                                        view! {
                                            <div class="font-mono text-gray-300">{file.clone()}</div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            // Money Section
                            <div class="border border-gray-600 p-3 rounded">
                                <h3 class="font-semibold mb-2">"Money"</h3>
                                <button
                                    class="px-2 py-1 bg-green-600 hover:bg-green-700 rounded text-sm mb-2"
                                    on:click=move |_| { check_money_action.dispatch(()); }
                                >
                                    "Check Balance"
                                </button>
                                {move || if server_money.get() > 0 {
                                    view! {
                                        <div>
                                            <div class="text-green-400 font-bold">"$"{server_money.get()}</div>
                                            <button
                                                class="mt-2 px-2 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-sm"
                                                on:click=move |_| {
                                                    transfer_money_action.dispatch(server_money.get().to_string());
                                                }
                                            >
                                                "Transfer All"
                                            </button>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }}
                            </div>

                            // Logs Section
                            <div class="border border-gray-600 p-3 rounded">
                                <h3 class="font-semibold mb-2">"Logs"</h3>
                                <div class="flex gap-2 mb-2">
                                    <button
                                        class="px-2 py-1 bg-gray-600 hover:bg-gray-700 rounded text-sm"
                                        on:click=move |_| { view_logs_action.dispatch(()); }
                                    >
                                        "View"
                                    </button>
                                    <button
                                        class="px-2 py-1 bg-red-600 hover:bg-red-700 rounded text-sm"
                                        on:click=move |_| { delete_logs_action.dispatch(()); }
                                    >
                                        "Delete"
                                    </button>
                                </div>
                                <div class="text-xs space-y-1 max-h-32 overflow-y-auto">
                                    {move || server_logs.get().iter().map(|log| {
                                        view! {
                                            <div class="text-gray-400">{log.clone()}</div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>
                    </div>
                }
            } else {
                view! { <div></div> }
            }}

            // Known Servers & Bounties
            <div class="grid grid-cols-2 gap-4">
                // Known Servers
                <div class="p-4 bg-gray-800 rounded">
                    <h2 class="text-xl font-bold mb-3">"Known Servers"</h2>
                    <div class="space-y-2">
                        {move || known_servers.get().iter().map(|server| {
                            view! {
                                <div class="p-2 bg-gray-700 rounded cursor-pointer hover:bg-gray-600"
                                     on:click=move |_| {
                                         set_target_ip.set(server.ip.clone());
                                     }>
                                    <div class="font-mono text-sm">{server.ip.clone()}</div>
                                    <div class="text-xs text-gray-400">{server.hostname.clone()}</div>
                                    <div class="text-xs text-gray-500">{server.notes.clone()}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Bounties
                <div class="p-4 bg-gray-800 rounded">
                    <h2 class="text-xl font-bold mb-3">"Available Bounties"</h2>
                    <div class="space-y-2">
                        {move || bounties.get().iter().map(|bounty| {
                            view! {
                                <div class="p-2 bg-gray-700 rounded">
                                    <div class="flex justify-between">
                                        <span class="font-semibold">{bounty.corporation.clone()}</span>
                                        <span class="text-green-400 font-bold">"$"{bounty.reward}</span>
                                    </div>
                                    <div class="text-sm">
                                        <span class="font-mono">{bounty.target_ip.clone()}</span>
                                        <span class="ml-2 text-xs text-yellow-400">
                                            {bounty.difficulty.clone()}
                                        </span>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}