//! Terms of Service page (1:1 port of TOS.php)
//! 
//! Displays the Terms of Service with internationalization support.
//! Original PHP: TOS.php

use axum::{
    extract::Query,
    http::HeaderMap,
    response::Html,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TosQuery {
    lang: Option<String>,
}

/// Terms of Service handler (TOS.php equivalent)
/// 
/// Displays the Terms of Service with language detection based on HOST header.
/// Defaults to en_US, switches to pt_BR for Brazilian subdomain.
pub async fn tos_handler(
    headers: HeaderMap,
    Query(_query): Query<TosQuery>,
) -> Html<String> {
    // Language detection based on HOST header (matches PHP logic)
    let lang = if let Some(host) = headers.get("host") {
        let host_str = host.to_str().unwrap_or("");
        if host_str.starts_with("br.") || host_str.starts_with("www.br.") {
            "pt_BR"
        } else {
            "en_US"
        }
    } else {
        "en_US"
    };

    // For now, we'll render the English version
    // TODO: Implement proper i18n with gettext-rs or similar
    let _ = lang; // Use variable to avoid warning

    let html = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Terms of Service</title>
<style>
body {
    margin-left: 100px;
    margin-right: 100px;
}

h1, h6 {
    text-align: center;
}

li {
    font-weight: bold;
}

ul {
    margin: 8px;
}
</style>
</head>
<body>

<h1>Terms of Service</h1>
<h6>Hacker Experience TOS</h6>

By using Hacker Experience, a service provided by NeoArt Labs, you are agreeing to be bound by the following terms and conditions. Violation of any of the terms of service which are stated below will result in temporary or permanent termination of your account. If you do not agree with any of these terms, you must not use Hacker Experience or any subdomain within www.hackerexperience.com.<br/><br/>

<li>1 - You agree</li>

<ul>1.1 - to respect every member of the game, regardless of gender, sexual orientation, sexual identity, religion, age, civil status, race, ethnicity or any other trait.</ul>
<ul>1.2 - to have only one account. Multiple accounts per IP are not allowed and will result on the termination of all of them. If you use a shared environment, contact us.</ul>
<ul>1.3 - to not post or publish offensive content on the forum, the in-game mail system, the logs or any other user-generated content system.</ul>
<ul>1.4 - to not exploit any vulnerability or system flaw you might encounter.</ul>
<ul>1.5 - that the use of tools to try to hack or (D)DoS the game is forbidden and legal measures might be taken.</ul>
<ul>1.6 - that you must not violate any law or make any unauthorized or illegal use while, and by, playing the game.</ul>
<ul>1.7 - that your use of Hacker Experience is at your sole risk. The service is provided on an "as is" basis.</ul>
<ul>1.8 - that NeoArt Labs does not warrant that the service will answer your needs, will be free of errors, will be secure or will be available at all times.</ul>
<ul>1.9 - and expressly understand that NeoArt Labs shall not be liable for any direct or indirect damages, including but not limited to damages for loss of data, profits, or other intangible losses resulting from the direct or indirect use of the service.</ul>
<ul>1.10 - that NeoArt Labs holds its rights to remove, or not to remove, any content which us unlawful or offensive.</ul>
<ul>1.11 - that Hacker Experience is a free service and it can be shutdown at any moment without prior notice.</ul>
<ul>1.12 - that you will be held responsible for every content and activity held under your account.</ul>
<ul>1.13 - with our <a target="__blank" href="privacy">Privacy Policy</a>.</ul>
<ul>1.14 - that any written abuse or threat made in an account will result in the immediate termination of that account.</ul>
<ul><strong>1.15 - that sharing non-public IPs on the forum is prohibited and might lead to account termination.</strong></ul>
<ul><strong>1.16 - that the use of any tool or script to gain unfair advantage over other players is forbidden.</strong></ul>
<ul>1.17 - that objects in mirror are closer than they appear.</ul>

<li>2 - You acknowledge that</li>

<ul>2.1 - this is a work of fiction. Names, characters, businesses, places, events and incidents are either the products of the author's imagination or used in a fictitious manner. Any resemblance to actual persons, living or dead, or actual events is purely coincidental.</ul>
<ul>2.2 - the game does not use any real hacking technique. It's entirely fictional.</ul>
<ul>2.3 - no potential real hacking knowledge can be learned from the game mechanics.</ul>

<li>3 - We guarantee that</li>

<ul>3.1 - every in-game IP is randomly generated and do not represent real user information.</ul>
<ul>3.2 - we do not spam.</ul>
<ul>3.3 - we do not sell or rent your data to anyone. (See <a target="__blank" href="privacy">Privacy Policy</a>)</ul>

<li>4 - When purchasing premium membership, you agree that</li>

<ul>4.1 - a valid credit card, which you have the right to use, is required for any paying account.</ul>
<ul>4.2 - the payments you make are non-refundable and are billed in advance. There will be no refunds of any sort or future credits for partial months usage of the service.</ul>
<ul>4.3 - all fees are exclusive of any kind of taxes, levies or duties imposed by taxing authorities.</ul>
<ul>4.4 - you will not be billed again if you wish to downgrade to the Basic account.</ul>
<ul>4.5 - price changes to the Premium Membership will affect only new upgrades from Basic to Premium plan. Current paying customers will keep the old price.</ul>

<li>5 - Account termination</li>

<ul>5.1 - Currently, the only way to cancel your account is by requesting it manually to contact@hackerexperience.com.</ul>
<ul>5.2 - If you are a recurring-paying premium member, you will not be billed after your account is terminated.</ul>
<ul>5.3 - NeoArt Labs has the right to terminate your account. This will result in the deactivation or deletion of your account and you will be prevented from any access to the game.</ul>
<ul><strong>5.4 - Due to limited personnel, deleting an account might take several days.</strong></ul>

<li>6 - Unenforceable provisions</li>

<ul>6.1 If any provision of this website disclaimer is, or is found to be, unenforceable under applicable law, that will not affect the enforceability of the other provisions of this website disclaimer.</ul>

<li>7 - Applicable law and competent court</li>

<ul>7.1 - Hacker Experience and NeoArt Labs are governed by Brazilian law. In case of disputes or arguments only the courts of Ribeirão Preto will be competent.</ul>
<ul>7.2 - In case of disputes a printed version of these terms and conditions of use will be accepted in legal or administrative procedures.</ul>

<li>8 - Changes to the TOS</li>

<ul>8.1 - NeoArt Labs reserves the right to update and change the Terms of Service from time to time without notice.</ul>
<ul>8.2 - Any changes or updates made to the application are subject to these Terms of Service.</ul>
<ul>8.3 - Continuing to use the service after such changes or updates are made will constitute your consent to those changes.</ul>
<ul>8.4 - Efforts will be made to publish major TOS changes on the Forum Announcements Board, but this does not invalidates item 8.1.</ul>

<li>Please do not sue me</li>
<ul>kthxbye</ul>

<br/><br/>

<center>
<strong>We use, follow and recommend the ACM code of ethics</strong><br/>
<a href="http://www.acm.org/about/code-of-ethics">http://www.acm.org/about/code-of-ethics</a>
<br/>
</center>

<br/><br/>

<center>
This Term of Service was first published on 30/08/2014 and last revised on 27/09/2014.
</center>
<br/>
</body>
</html>"#;

    Html(html.to_string())
}