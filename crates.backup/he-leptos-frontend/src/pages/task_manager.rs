use leptos::*;

#[component]
pub fn TaskManagerPage() -> impl IntoView {
    view! {
        <div>
            // Page header
            <div class="page-header" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 10px 15px; margin-bottom: 20px;">
                <h2 style="font-size: 14px; margin: 0; color: #00ff00; text-shadow: 0 0 5px #00ff00; display: inline-block;">
                    "Task Manager"
                </h2>
                <span class="ip-address" style="float: right; font-family: monospace; font-size: 14px; color: #00ff00; text-shadow: 0 0 5px #00ff00;">
                    "[192.168.1.42]"
                </span>
            </div>

            // Tab navigation
            <div style="border-bottom: 1px solid #333; margin-bottom: 20px;">
                <ul style="list-style: none; margin: 0; padding: 0; display: flex;">
                    <li style="margin-right: 10px;">
                        <a href="#active" style="display: inline-block; padding: 8px 15px; background: #1a1a1a; border: 1px solid #00ff00; border-bottom: none; color: #00ff00 !important; text-decoration: none; font-size: 11px;">
                            "Active Processes"
                        </a>
                    </li>
                    <li style="margin-right: 10px;">
                        <a href="#completed" style="display: inline-block; padding: 8px 15px; background: #0a0a0a; border: 1px solid #333; border-bottom: none; color: #888888 !important; text-decoration: none; font-size: 11px;">
                            "Completed"
                        </a>
                    </li>
                    <li>
                        <a href="#failed" style="display: inline-block; padding: 8px 15px; background: #0a0a0a; border: 1px solid #333; border-bottom: none; color: #888888 !important; text-decoration: none; font-size: 11px;">
                            "Failed"
                        </a>
                    </li>
                </ul>
            </div>

            // Active processes
            <div class="panel" style="background: #0a0a0a; border: 1px solid #333333; margin-bottom: 20px;">
                <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                    <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                        "Active Processes"
                    </h3>
                </div>
                <div class="panel-body" style="padding: 12px;">
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Process"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Target"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Progress"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Time Remaining"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: center; font-size: 11px; color: #00ff00;">
                                    "Action"
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "Running Cracker v3.0"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888; font-family: monospace;">
                                    "192.168.50.23"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222;">
                                    <div style="height: 16px; background: #0a0a0a; border: 1px solid #333333; border-radius: 2px; overflow: hidden;">
                                        <div style="height: 100%; width: 45%; background: linear-gradient(to right, #003300, #00ff00); transition: width 0.6s ease;">
                                            <span style="display: block; text-align: center; font-size: 10px; line-height: 16px; color: #000000;">
                                                "45%"
                                            </span>
                                        </div>
                                    </div>
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "02:15"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; text-align: center;">
                                    <button style="padding: 2px 6px; font-size: 10px; background: linear-gradient(to bottom, #330000, #110000); border: 1px solid #ff0000; color: #ff6666; cursor: pointer;">
                                        "Cancel"
                                    </button>
                                </td>
                            </tr>
                            <tr>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "Downloading Files"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888; font-family: monospace;">
                                    "45.23.176.9"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222;">
                                    <div style="height: 16px; background: #0a0a0a; border: 1px solid #333333; border-radius: 2px; overflow: hidden;">
                                        <div style="height: 100%; width: 78%; background: linear-gradient(to right, #003300, #00ff00); transition: width 0.6s ease;">
                                            <span style="display: block; text-align: center; font-size: 10px; line-height: 16px; color: #000000;">
                                                "78%"
                                            </span>
                                        </div>
                                    </div>
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "00:45"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; text-align: center;">
                                    <button style="padding: 2px 6px; font-size: 10px; background: linear-gradient(to bottom, #330000, #110000); border: 1px solid #ff0000; color: #ff6666; cursor: pointer;">
                                        "Cancel"
                                    </button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            // Running software
            <div class="panel" style="background: #0a0a0a; border: 1px solid #333333;">
                <div class="panel-heading" style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border-bottom: 1px solid #00ff00; padding: 8px 12px;">
                    <h3 class="panel-title" style="font-size: 12px; margin: 0; color: #00ff00; text-shadow: 0 0 3px #00ff00;">
                        "Running Software"
                    </h3>
                </div>
                <div class="panel-body" style="padding: 12px;">
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Software"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "Version"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "CPU Usage"
                                </th>
                                <th style="background: linear-gradient(to bottom, #2a2a2a, #1a1a1a); border: 1px solid #333333; padding: 6px; text-align: left; font-size: 11px; color: #00ff00;">
                                    "RAM Usage"
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "Firewall"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "5.2"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "250 MHz"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "512 MB"
                                </td>
                            </tr>
                            <tr>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "Hasher"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "3.0"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "100 MHz"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "256 MB"
                                </td>
                            </tr>
                            <tr>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "Seeker"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #888888;">
                                    "2.1"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "50 MHz"
                                </td>
                                <td style="padding: 6px; border: 1px solid #222222; font-size: 11px; color: #ffaa00;">
                                    "128 MB"
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}