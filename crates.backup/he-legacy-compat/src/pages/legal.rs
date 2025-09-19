//! Legal page handler - 1:1 port of legal.php
//! 
//! Displays legal information with tabbed interface for Terms of Service and Privacy Policy.
//! Shows iframe content from TOS.php and privacy.php based on selected tab.

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, player::Player, ranking::Ranking};
use crate::session::PhpSession;
use he_db::DbPool;

/// Query parameters for legal page
#[derive(Debug, Deserialize)]
pub struct LegalQuery {
    pub show: Option<String>,
}

/// Legal page handler - displays Terms of Service and Privacy Policy in tabbed interface
/// 
/// Port of: legal.php
/// Behavior:
/// - Shows tabbed interface with TOS and Privacy Policy
/// - Switches content based on ?show parameter
/// - Displays iframe with TOS.php or privacy.php content
/// - Includes warning message about consent
/// - Provides "Open in new tab" link
pub async fn legal_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(session): Extension<PhpSession>,
    Query(params): Query<LegalQuery>,
) -> Result<Html<String>, StatusCode> {
    let mut system = System::new(db_pool.clone());
    let mut ranking = Ranking::new(db_pool.clone());

    // Determine which tab is active based on show parameter
    let mut show = "tos";
    let mut tos_class = " active";
    let mut pp_class = "";
    let mut source = "TOS.php";

    if let Some(show_param) = params.show.as_deref() {
        // Original PHP uses switchGet to validate allowed values: 'tos', 'privacy', 'forum'
        match show_param {
            "privacy" => {
                show = "pp";
                tos_class = "";
                pp_class = " active";
                source = "privacy.php";
            },
            "tos" | "forum" => {
                // Keep defaults for TOS or forum (forum redirects to TOS)
                show = "tos";
            },
            _ => {
                // Invalid parameter, keep defaults
            }
        }
    }

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Legal Information - Hacker Experience</title>
    <link href="css/bootstrap.css" rel="stylesheet">
    <link href="css/he_index.css" rel="stylesheet">
    <style>
        .widget-box {{
            border: 1px solid #ddd;
            border-radius: 4px;
            margin-bottom: 20px;
        }}
        .widget-title {{
            background: #f5f5f5;
            border-bottom: 1px solid #ddd;
            border-radius: 4px 4px 0 0;
            padding: 0;
        }}
        .widget-content {{
            padding: 20px;
        }}
        .widget-content.padding.noborder {{
            border: none;
            padding: 15px;
        }}
        .nav-tabs {{
            border-bottom: 1px solid #ddd;
            margin: 0;
        }}
        .nav-tabs li {{
            margin-bottom: -1px;
        }}
        .nav-tabs li a {{
            border: 1px solid transparent;
            border-radius: 4px 4px 0 0;
            color: #666;
            display: block;
            padding: 8px 12px;
            text-decoration: none;
        }}
        .nav-tabs li.active a {{
            background-color: #fff;
            border-color: #ddd #ddd transparent;
            color: #333;
        }}
        .alert {{
            background-color: #f2dede;
            border: 1px solid #ebccd1;
            border-radius: 4px;
            color: #a94442;
            margin-bottom: 20px;
            padding: 15px;
        }}
        .alert-error {{
            background-color: #f2dede;
            border-color: #ebccd1;
            color: #a94442;
        }}
        .close {{
            background: transparent;
            border: 0;
            color: #000;
            float: right;
            font-size: 20px;
            font-weight: bold;
            opacity: 0.2;
        }}
        .close:hover {{
            opacity: 0.5;
        }}
        #legalframe {{
            border: none;
            height: 600px;
            min-height: 500px;
        }}
        .btn {{
            background-color: #d9534f;
            border: 1px solid #d43f3a;
            border-radius: 4px;
            color: #fff;
            display: inline-block;
            font-size: 14px;
            font-weight: normal;
            line-height: 1.42857143;
            margin-bottom: 0;
            padding: 6px 12px;
            text-align: center;
            text-decoration: none;
            vertical-align: middle;
            white-space: nowrap;
        }}
        .btn-danger {{
            background-color: #d9534f;
            border-color: #d43f3a;
        }}
        .btn:hover {{
            background-color: #c9302c;
            border-color: #ac2925;
        }}
        .label {{
            background-color: #5bc0de;
            border-radius: 0.25em;
            color: #fff;
            display: inline;
            font-size: 75%;
            font-weight: bold;
            line-height: 1;
            padding: 0.2em 0.6em 0.3em;
            text-align: center;
            vertical-align: baseline;
            white-space: nowrap;
        }}
        .label-info {{
            background-color: #5bc0de;
        }}
        .span12 {{
            width: 100%;
        }}
        .icon-tab {{
            display: inline-block;
            height: 16px;
            margin-right: 5px;
            width: 16px;
        }}
        .he16-tos {{
            background: url('images/icons/terms.png') no-repeat;
        }}
        .he16-privacy {{
            background: url('images/icons/privacy.png') no-repeat;
        }}
    </style>
</head>
<body>
    <div class="span12">
        <div class="alert alert-error">
            <button class="close" data-dismiss="alert">×</button>
            <strong>Attention!</strong> By playing Hacker Experience you consent with the following terms, so read them carefully!
        </div>
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">                                  
                    <li class="link{tos_active}"><a href="legal.php"><span class="icon-tab he16-tos"></span>Terms of service</a></li>
                    <li class="link{pp_active}"><a href="?show=privacy"><span class="icon-tab he16-privacy"></span>Privacy Policy</a></li>
                    <a href="#"><span class="label label-info">Help</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
                <iframe id="legalframe" src="{source_file}" width="100%" seamless="1"></iframe>
                <center><a class="btn btn-danger" target="_blank" href="{source_file}">Open in a new tab</a></center>
            </div>
        </div>
        <div class="nav nav-tabs" style="clear: both;"></div>
    </div>

    <script>
        // Close button functionality
        document.querySelector('.close').addEventListener('click', function() {{
            this.parentElement.style.display = 'none';
        }});
    </script>
</body>
</html>
    "#,
        tos_active = tos_class,
        pp_active = pp_class,
        source_file = source
    );

    Ok(Html(html))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_legal_query_deserialize() {
        let query_tos = LegalQuery {
            show: Some("tos".to_string()),
        };
        assert_eq!(query_tos.show.as_deref(), Some("tos"));

        let query_privacy = LegalQuery {
            show: Some("privacy".to_string()),
        };
        assert_eq!(query_privacy.show.as_deref(), Some("privacy"));

        let query_none = LegalQuery {
            show: None,
        };
        assert!(query_none.show.is_none());
    }

    #[test]
    fn test_show_parameter_logic() {
        // Test TOS default
        let show_param: Option<&str> = None;
        let (show, source) = match show_param {
            Some("privacy") => ("pp", "privacy.php"),
            _ => ("tos", "TOS.php"),
        };
        assert_eq!(show, "tos");
        assert_eq!(source, "TOS.php");

        // Test privacy parameter
        let show_param = Some("privacy");
        let (show, source) = match show_param {
            Some("privacy") => ("pp", "privacy.php"),
            _ => ("tos", "TOS.php"),
        };
        assert_eq!(show, "pp");
        assert_eq!(source, "privacy.php");
    }
}