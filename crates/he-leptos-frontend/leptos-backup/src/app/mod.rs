use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::*;
use crate::components::*;
use crate::state::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(create_game_state());

    view! {
        <Title text="Control Panel - NetHeist"/>
        <Router>
            // Header
            <div id="header">
                <h1><A href="/">"NetHeist"</A></h1>
            </div>

            // User navigation bar  
            <div id="user-nav" class="navbar navbar-inverse">
                <ul class="nav btn-group">
                    <li class="btn btn-inverse">
                        <A href="/profile"><i class="fa fa-user"></i> <span class="text">"Techmad"</span></A>
                    </li>
                    <li class="btn btn-inverse">
                        <A href="/mail"><i class="fa fa-envelope"></i> <span class="text">"E-Mail"</span> <span class="mail-unread">"2"</span></A>
                    </li>
                    <li class="btn btn-inverse">
                        <A href="/settings"><i class="fa fa-wrench"></i> <span class="text">"Settings"</span></A>
                    </li>
                    <li class="btn btn-inverse">
                        <A href="/logout"><i class="fa fa-power-off"></i> <span class="text">"Logout"</span></A>
                    </li>
                </ul>
            </div>

            // Sidebar
            <div id="sidebar">
                <ul>
                    <li class="active"><A href="/"><i class="fa fa-home"></i> <span>"Home"</span></A></li>
                    <li><A href="/task-manager"><i class="fa fa-tasks"></i> <span>"Task Manager"</span></A></li>
                    <li id="menu-software"><A href="/software"><i class="fa fa-folder-open"></i> <span>"Software"</span></A></li>
                    <li id="menu-internet"><A href="/internet"><i class="fa fa-globe"></i> <span>"Internet"</span></A></li>
                    <li><A href="/logs"><i class="fa fa-file-text"></i> <span>"Log File"</span></A></li>
                    <li><A href="/missions"><i class="fa fa-flag"></i> <span>"Missions"</span></A></li>
                    <li><A href="/finances"><i class="fa fa-dollar"></i> <span>"Finances"</span></A></li>
                    <li><A href="/clan"><i class="fa fa-users"></i> <span>"Clan"</span></A></li>
                    <li><A href="/ranking"><i class="fa fa-trophy"></i> <span>"Ranking"</span></A></li>
                    <li><A href="/hardware"><i class="fa fa-desktop"></i> <span>"Hardware"</span></A></li>
                    <li><A href="/university"><i class="fa fa-graduation-cap"></i> <span>"University"</span></A></li>
                    <li><A href="/utilities"><i class="fa fa-wrench"></i> <span>"Utilities"</span></A></li>
                </ul>
            </div>

            // Content
            <div id="content">
                <div id="content-header">
                    <h1>"Home"</h1>
                    <div class="header-ip">
                        <span class="header-ip-show">"192.168.1.100"</span>
                    </div>
                    <div class="header-info">
                        <span class="small">"Location: "</span><span class="green">"Local"</span>
                        <span class="small">" | OS: "</span><span class="green">"Linux"</span>
                        <span class="small">" | Speed: "</span><span class="green">"1000 Mbps"</span>
                    </div>
                </div>

                <div id="breadcrumb">
                    <A href="/">"Home"</A>
                </div>

                <div class="container-fluid">
                    <Routes>
                        <Route path="" view=HomePage/>
                        <Route path="/task-manager" view=TaskManagerPage/>
                        <Route path="/software" view=SoftwarePage/>
                        <Route path="/internet" view=InternetPage/>
                        <Route path="/logs" view=LogsPage/>
                        <Route path="/missions" view=MissionsPage/>
                        <Route path="/finances" view=FinancesPage/>
                        <Route path="/clan" view=ClanPage/>
                        <Route path="/ranking" view=RankingPage/>
                        <Route path="/hardware" view=HardwarePage/>
                        <Route path="/university" view=UniversityPage/>
                        <Route path="/utilities" view=UtilitiesPage/>
                        <Route path="/*any" view=NotFound/>
                    </Routes>
                </div>
            </div>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div style="text-align: center; padding: 50px;">
            <h1 style="color: #ff0000; text-shadow: 0 0 10px #ff0000;">"404 - Page Not Found"</h1>
            <p style="color: #888888; margin-top: 20px;">"The page you are looking for does not exist."</p>
            <a href="/" style="color: #00ff00 !important; text-decoration: none; display: inline-block; margin-top: 20px;">
                "Return to Control Panel"
            </a>
        </div>
    }
}