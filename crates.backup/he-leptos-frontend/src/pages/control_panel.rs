use leptos::*;

#[component]
pub fn ControlPanelPage() -> impl IntoView {
    view! {
        <div>
            // Page header
            <div class="page-header" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 10px 15px; margin-bottom: 20px;">
                <h2 style="font-size: 14px; margin: 0; color: #00ff00; text-shadow: 0 0 5px #00ff00; display: inline-block;">
                    "Control Panel"
                </h2>
                <span class="ip-address" style="float: right; font-family: monospace; font-size: 14px; color: #00ff00; text-shadow: 0 0 5px #00ff00;">
                    "[192.168.1.42]"
                </span>
            </div>

            // Main content grid
            <div style="display: flex; gap: 20px;">
                // Left column
                <div style="flex: 0 0 25%;">
                    // Hardware Info Widget
                    <div class="panel" style="background: #0a0a0a; border: 1px solid #333333; margin-bottom: 20px;">
                        <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                            <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                                "Hardware Info"
                            </h3>
                        </div>
                        <div class="panel-body" style="padding: 12px; color: #c0c0c0;">
                            <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #888888; font-size: 11px;">"CPU:"</span>
                                <span style="color: #00ff00; font-size: 11px;">"4.0 GHz"</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #888888; font-size: 11px;">"HDD:"</span>
                                <span style="color: #00ff00; font-size: 11px;">"2 TB"</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #888888; font-size: 11px;">"RAM:"</span>
                                <span style="color: #00ff00; font-size: 11px;">"16 GB"</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #888888; font-size: 11px;">"NET:"</span>
                                <span style="color: #00ff00; font-size: 11px;">"1 Gbit/s"</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; padding: 5px 0;">
                                <span style="color: #888888; font-size: 11px;">"External HD:"</span>
                                <span style="color: #00ff00; font-size: 11px;">"None"</span>
                            </div>
                        </div>
                    </div>

                    // General Info Widget
                    <div class="panel" style="background: #0a0a0a; border: 1px solid #333333; margin-bottom: 20px;">
                        <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                            <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                                "General Info"
                            </h3>
                        </div>
                        <div class="panel-body" style="padding: 12px; color: #c0c0c0;">
                            <div style="margin-bottom: 10px;">
                                <span style="color: #888888; font-size: 11px; display: inline-block; width: 100px;">"Username:"</span>
                                <span style="color: #c0c0c0; font-size: 11px;">"EliteH4x0r"</span>
                            </div>
                            <div style="margin-bottom: 10px;">
                                <span style="color: #888888; font-size: 11px; display: inline-block; width: 100px;">"Reputation:"</span>
                                <span style="color: #00ff00; font-size: 11px; font-weight: bold;">"15,234"</span>
                            </div>
                            <div style="margin-bottom: 10px;">
                                <span style="color: #888888; font-size: 11px; display: inline-block; width: 100px;">"Ranking:"</span>
                                <span style="color: #ff0000; font-size: 11px; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"#42"</span>
                            </div>
                            <div style="margin-bottom: 10px;">
                                <span style="color: #888888; font-size: 11px; display: inline-block; width: 100px;">"Clan:"</span>
                                <span style="color: #ff0000; font-size: 11px; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"[ELITE]"</span>
                            </div>
                            <div>
                                <span style="color: #888888; font-size: 11px; display: inline-block; width: 100px;">"Online since:"</span>
                                <span style="color: #c0c0c0; font-size: 11px;">"2024-01-15"</span>
                            </div>
                        </div>
                    </div>
                </div>

                // Middle column
                <div style="flex: 0 0 50%;">
                    // News Widget
                    <div class="panel" style="background: #0a0a0a; border: 1px solid #333333; margin-bottom: 20px;">
                        <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                            <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                                "Latest News"
                            </h3>
                        </div>
                        <div class="panel-body" style="padding: 12px;">
                            <div style="padding: 8px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #666666; font-size: 10px; margin-right: 10px;">"2024-09-14"</span>
                                <a href="#" style="color: #888888 !important; font-size: 11px; text-decoration: none;">
                                    "New tournament announced - Grand Hacking Challenge"
                                </a>
                            </div>
                            <div style="padding: 8px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #666666; font-size: 10px; margin-right: 10px;">"2024-09-13"</span>
                                <a href="#" style="color: #888888 !important; font-size: 11px; text-decoration: none;">
                                    "Server maintenance completed"
                                </a>
                            </div>
                            <div style="padding: 8px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #666666; font-size: 10px; margin-right: 10px;">"2024-09-12"</span>
                                <a href="#" style="color: #888888 !important; font-size: 11px; text-decoration: none;">
                                    "New clan system updates"
                                </a>
                            </div>
                            <div style="padding: 8px 0; border-bottom: 1px solid #222222;">
                                <span style="color: #666666; font-size: 10px; margin-right: 10px;">"2024-09-11"</span>
                                <a href="#" style="color: #888888 !important; font-size: 11px; text-decoration: none;">
                                    "DDoS protection enhanced"
                                </a>
                            </div>
                            <div style="padding: 8px 0;">
                                <span style="color: #666666; font-size: 10px; margin-right: 10px;">"2024-09-10"</span>
                                <a href="#" style="color: #888888 !important; font-size: 11px; text-decoration: none;">
                                    "New mission pack available"
                                </a>
                            </div>
                        </div>
                    </div>

                    // FBI Most Wanted
                    <div class="panel" style="background: #0a0a0a; border: 1px solid #333333;">
                        <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                            <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                                "FBI Most Wanted List"
                            </h3>
                        </div>
                        <div class="panel-body" style="padding: 12px;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead>
                                    <tr>
                                        <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">"#"</th>
                                        <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">"Hacker"</th>
                                        <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">"Bounty"</th>
                                        <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">"IP"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ff0000; font-weight: bold;">"1"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">"DarkPhoenix"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #00ff00;">"$50,000"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888; font-family: monospace;">"45.23.176.9"</td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ff0000; font-weight: bold;">"2"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">"ZeroCool"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #00ff00;">"$35,000"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888; font-family: monospace;">"92.14.88.231"</td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ff0000; font-weight: bold;">"3"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">"CrashOverride"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #00ff00;">"$25,000"</td>
                                        <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888; font-family: monospace;">"178.92.44.156"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>

                // Right column
                <div style="flex: 0 0 25%;">
                    // Top Rankings
                    <div class="panel" style="background: #0a0a0a; border: 1px solid #333333;">
                        <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                            <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                                "Top Rankings"
                            </h3>
                        </div>
                        <div class="panel-body" style="padding: 12px;">
                            <table style="width: 100%;">
                                <tbody>
                                    <tr>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <span style="color: #ff0000; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"1."</span>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <a href="#" style="color: #888888 !important; text-decoration: none;">"NeoMatrix"</a>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222; text-align: right;">
                                            <span style="color: #00ff00; font-weight: bold;">"98,542"</span>
                                        </td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <span style="color: #ff0000; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"2."</span>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <a href="#" style="color: #888888 !important; text-decoration: none;">"CyberGhost"</a>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222; text-align: right;">
                                            <span style="color: #00ff00; font-weight: bold;">"87,234"</span>
                                        </td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <span style="color: #ff0000; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"3."</span>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <a href="#" style="color: #888888 !important; text-decoration: none;">"PhantomByte"</a>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222; text-align: right;">
                                            <span style="color: #00ff00; font-weight: bold;">"76,891"</span>
                                        </td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <span style="color: #ff0000; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"4."</span>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222;">
                                            <a href="#" style="color: #888888 !important; text-decoration: none;">"ShadowNet"</a>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; border-bottom: 1px solid #222222; text-align: right;">
                                            <span style="color: #00ff00; font-weight: bold;">"65,432"</span>
                                        </td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 5px 10px; font-size: 11px;">
                                            <span style="color: #ff0000; font-weight: bold; text-shadow: 0 0 3px #ff0000;">"5."</span>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px;">
                                            <a href="#" style="color: #888888 !important; text-decoration: none;">"VirusKing"</a>
                                        </td>
                                        <td style="padding: 5px 10px; font-size: 11px; text-align: right;">
                                            <span style="color: #00ff00; font-weight: bold;">"54,876"</span>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}