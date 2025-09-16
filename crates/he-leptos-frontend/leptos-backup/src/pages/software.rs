use leptos::*;
use leptos::ev::MouseEvent;

#[derive(Clone, Debug, PartialEq)]
struct Software {
    name: String,
    version: String,
    size: f32,
    actions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum StorageType {
    Local,
    External,
}

#[component]
pub fn SoftwarePage() -> impl IntoView {
    // State for current storage type
    let (storage_type, set_storage_type) = create_signal(StorageType::Local);
    
    // Local software list
    let local_software = vec![
        Software {
            name: "Cracker".to_string(),
            version: "1.0".to_string(),
            size: 1.0,
            actions: vec!["Run".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Hasher".to_string(),
            version: "0.5".to_string(),
            size: 1.5,
            actions: vec!["Run".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "SSH Exploit".to_string(),
            version: "2.0".to_string(),
            size: 2.0,
            actions: vec!["Run".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "FTP Exploit".to_string(),
            version: "1.5".to_string(),
            size: 1.8,
            actions: vec!["Run".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Firewall".to_string(),
            version: "3.0".to_string(),
            size: 5.0,
            actions: vec!["Install".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Antivirus".to_string(),
            version: "2.5".to_string(),
            size: 4.5,
            actions: vec!["Install".to_string(), "Delete".to_string()],
        },
        Software {
            name: "DDoS".to_string(),
            version: "1.0".to_string(),
            size: 3.0,
            actions: vec!["Run".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Virus".to_string(),
            version: "0.8".to_string(),
            size: 0.5,
            actions: vec!["Upload".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Worm".to_string(),
            version: "1.2".to_string(),
            size: 0.8,
            actions: vec!["Upload".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Spam Bot".to_string(),
            version: "1.0".to_string(),
            size: 1.2,
            actions: vec!["Upload".to_string(), "Hide".to_string(), "Delete".to_string()],
        },
    ];
    
    // External software (less items)
    let external_software = vec![
        Software {
            name: "Backup Firewall".to_string(),
            version: "2.0".to_string(),
            size: 4.0,
            actions: vec!["Move to Local".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Old Cracker".to_string(),
            version: "0.8".to_string(),
            size: 0.8,
            actions: vec!["Move to Local".to_string(), "Delete".to_string()],
        },
        Software {
            name: "Research Notes".to_string(),
            version: "N/A".to_string(),
            size: 0.1,
            actions: vec!["View".to_string(), "Delete".to_string()],
        },
    ];
    
    // Calculate total and used storage
    let (local_used, _) = create_signal(
        local_software.iter().map(|s| s.size).sum::<f32>()
    );
    let (external_used, _) = create_signal(
        external_software.iter().map(|s| s.size).sum::<f32>()
    );
    
    let local_total = 500.0; // GB
    let external_total = 1000.0; // GB
    
    // Handle software actions
    let handle_action = move |software_name: String, action: String| {
        web_sys::window()
            .unwrap()
            .alert_with_message(&format!("{} on {}", action, software_name))
            .unwrap();
    };
    
    view! {
        <div>
            // Content header
            <div id="content-header">
                <h1>"My Software"</h1>
                <div class="header-ip">
                    <span class="header-ip-show">"localhost"</span>
                </div>
            </div>
            
            // Breadcrumb
            <div id="breadcrumb">
                <a href="#" title="Go to Home" class="tip-bottom">"Home"</a>" / "
                <a href="#" class="current">"Software"</a>
            </div>
            
            // Main content
            <div class="container-fluid">
                <div class="row-fluid">
                    <div class="span12">
                        <div class="widget-box">
                            <div class="widget-title">
                                // Tab navigation
                                <ul class="nav nav-tabs">
                                    <li class=move || if storage_type() == StorageType::Local { "active" } else { "" }>
                                        <a 
                                            href="#"
                                            on:click=move |e: MouseEvent| {
                                                e.prevent_default();
                                                set_storage_type(StorageType::Local);
                                            }
                                        >
                                            <i class="fa fa-hdd-o"></i>" Local HD"
                                        </a>
                                    </li>
                                    <li class=move || if storage_type() == StorageType::External { "active" } else { "" }>
                                        <a 
                                            href="#"
                                            on:click=move |e: MouseEvent| {
                                                e.prevent_default();
                                                set_storage_type(StorageType::External);
                                            }
                                        >
                                            <i class="fa fa-usb"></i>" External HD"
                                        </a>
                                    </li>
                                </ul>
                            </div>
                            
                            <div class="widget-content">
                                // Storage info
                                <div style="margin-bottom: 20px;">
                                    {move || match storage_type() {
                                        StorageType::Local => view! {
                                            <div>
                                                <strong>"Storage: "</strong>
                                                {local_used()}" GB / "{local_total}" GB"
                                                <div class="progress" style="margin: 10px 0;">
                                                    <div 
                                                        class="progress-bar"
                                                        style=move || format!("width: {}%", (local_used() / local_total * 100.0))
                                                    >
                                                        {move || format!("{:.1}%", (local_used() / local_total * 100.0))}
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_view(),
                                        StorageType::External => view! {
                                            <div>
                                                <strong>"Storage: "</strong>
                                                {external_used()}" GB / "{external_total}" GB"
                                                <div class="progress" style="margin: 10px 0;">
                                                    <div 
                                                        class="progress-bar"
                                                        style=move || format!("width: {}%", (external_used() / external_total * 100.0))
                                                    >
                                                        {move || format!("{:.1}%", (external_used() / external_total * 100.0))}
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_view(),
                                    }}
                                </div>
                                
                                // Software table
                                <table class="table table-bordered table-striped">
                                    <thead>
                                        <tr>
                                            <th>"Software"</th>
                                            <th>"Version"</th>
                                            <th>"Size (GB)"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || {
                                            let software_list = match storage_type() {
                                                StorageType::Local => local_software.clone(),
                                                StorageType::External => external_software.clone(),
                                            };
                                            
                                            software_list.into_iter().map(|software| {
                                                let name_clone = software.name.clone();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <i class="heicon"></i>
                                                            <span class="item">{&software.name}</span>
                                                        </td>
                                                        <td>{software.version}</td>
                                                        <td>{software.size}</td>
                                                        <td>
                                                            {software.actions.into_iter().map(|action| {
                                                                let action_clone = action.clone();
                                                                let name_for_action = name_clone.clone();
                                                                view! {
                                                                    <button 
                                                                        class="btn btn-mini"
                                                                        style="margin-right: 5px;"
                                                                        on:click=move |_| {
                                                                            handle_action(name_for_action.clone(), action_clone.clone())
                                                                        }
                                                                    >
                                                                        {action}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        }}
                                    </tbody>
                                </table>
                                
                                // Action buttons
                                <div style="margin-top: 20px;">
                                    <button class="btn btn-primary">
                                        <i class="fa fa-download"></i>" Download Software"
                                    </button>
                                    <button class="btn" style="margin-left: 10px;">
                                        <i class="fa fa-code"></i>" Research New Software"
                                    </button>
                                    <button class="btn" style="margin-left: 10px;">
                                        <i class="fa fa-shield"></i>" Run Full Scan"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
