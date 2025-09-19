//! Marketplace Page - Buy and sell software, exploits, and services

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MarketItem {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub seller: String,
    pub price: u64,
    pub rating: f32,
    pub downloads: u32,
    pub description: String,
    pub version: String,
}

#[component]
pub fn MarketplacePage() -> impl IntoView {
    let (items, set_items) = create_signal(vec![
        MarketItem {
            id: 1,
            name: "Zero-Day Exploit Pack".to_string(),
            category: "Exploits".to_string(),
            seller: "DarkCoder".to_string(),
            price: 100000,
            rating: 4.8,
            downloads: 523,
            description: "Collection of unreleased zero-day exploits".to_string(),
            version: "2025.1".to_string(),
        },
        MarketItem {
            id: 2,
            name: "Quantum Firewall Breaker".to_string(),
            category: "Crackers".to_string(),
            seller: "CryptoMaster".to_string(),
            price: 75000,
            rating: 4.5,
            downloads: 342,
            description: "Advanced firewall penetration tool".to_string(),
            version: "3.0".to_string(),
        },
        MarketItem {
            id: 3,
            name: "Banking Trojan Kit".to_string(),
            category: "Viruses".to_string(),
            seller: "PhantomHacker".to_string(),
            price: 150000,
            rating: 4.9,
            downloads: 187,
            description: "Sophisticated banking malware framework".to_string(),
            version: "5.2".to_string(),
        },
        MarketItem {
            id: 4,
            name: "Neural Network Scanner".to_string(),
            category: "Tools".to_string(),
            seller: "AIHacker".to_string(),
            price: 50000,
            rating: 4.3,
            downloads: 892,
            description: "AI-powered vulnerability scanner".to_string(),
            version: "1.5".to_string(),
        },
        MarketItem {
            id: 5,
            name: "Stealth VPN Service".to_string(),
            category: "Services".to_string(),
            seller: "GhostNet".to_string(),
            price: 10000,
            rating: 4.7,
            downloads: 2341,
            description: "Untraceable VPN with military encryption".to_string(),
            version: "Pro".to_string(),
        },
    ]);

    let (selected_category, set_selected_category) = create_signal("All");
    let (search_query, set_search_query) = create_signal(String::new());
    let (sort_by, set_sort_by) = create_signal("popular");
    let (cart, set_cart) = create_signal(Vec::<u32>::new());

    let filtered_items = move || {
        let category = selected_category.get();
        let query = search_query.get().to_lowercase();
        let mut items_filtered: Vec<_> = items.get().into_iter().filter(|item| {
            (category == "All" || item.category == category) &&
            (query.is_empty() ||
             item.name.to_lowercase().contains(&query) ||
             item.description.to_lowercase().contains(&query))
        }).collect();

        match sort_by.get() {
            "price-low" => items_filtered.sort_by_key(|i| i.price),
            "price-high" => items_filtered.sort_by_key(|i| std::cmp::Reverse(i.price)),
            "rating" => items_filtered.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
            _ => items_filtered.sort_by_key(|i| std::cmp::Reverse(i.downloads)),
        }

        items_filtered
    };

    let cart_total = move || {
        cart.get().iter().filter_map(|id| {
            items.get().iter().find(|i| i.id == *id).map(|i| i.price)
        }).sum::<u64>()
    };

    view! {
        <div class="marketplace-page">
            <div class="marketplace-header">
                <h1>"Underground Marketplace"</h1>
                <div class="cart-info">
                    <span class="cart-icon">"🛒"</span>
                    <span class="cart-count">{move || cart.get().len()}</span>
                    <span class="cart-total">"$"{cart_total()}</span>
                </div>
            </div>

            <div class="marketplace-controls">
                <div class="search-bar">
                    <input
                        type="text"
                        placeholder="Search marketplace..."
                        class="search-input"
                        on:input=move |ev| set_search_query(event_target_value(&ev))
                    />
                </div>
                <div class="filter-sort">
                    <select
                        class="category-filter"
                        on:change=move |ev| set_selected_category(event_target_value(&ev))
                    >
                        <option value="All">"All Categories"</option>
                        <option value="Exploits">"Exploits"</option>
                        <option value="Crackers">"Crackers"</option>
                        <option value="Viruses">"Viruses"</option>
                        <option value="Tools">"Tools"</option>
                        <option value="Services">"Services"</option>
                    </select>
                    <select
                        class="sort-filter"
                        on:change=move |ev| set_sort_by(event_target_value(&ev))
                    >
                        <option value="popular">"Most Popular"</option>
                        <option value="rating">"Highest Rated"</option>
                        <option value="price-low">"Price: Low to High"</option>
                        <option value="price-high">"Price: High to Low"</option>
                    </select>
                </div>
            </div>

            <div class="marketplace-grid">
                {move || filtered_items().into_iter().map(|item| {
                    let item_id = item.id;
                    let is_in_cart = move || cart.get().contains(&item_id);

                    view! {
                        <div class="market-item">
                            <div class="item-header">
                                <span class="item-category">{&item.category}</span>
                                <span class="item-version">"v"{&item.version}</span>
                            </div>
                            <h3 class="item-name">{&item.name}</h3>
                            <p class="item-description">{&item.description}</p>
                            <div class="item-seller">
                                <span>"By: "</span>
                                <strong>{&item.seller}</strong>
                            </div>
                            <div class="item-stats">
                                <div class="item-rating">
                                    <span class="stars">
                                        {(0..5).map(|i| {
                                            if i < item.rating as usize {
                                                "★"
                                            } else {
                                                "☆"
                                            }
                                        }).collect::<String>()}
                                    </span>
                                    <span class="rating-value">{item.rating}</span>
                                </div>
                                <div class="item-downloads">
                                    <span>"📥 "{item.downloads}</span>
                                </div>
                            </div>
                            <div class="item-footer">
                                <span class="item-price">"$"{item.price}</span>
                                {move || if is_in_cart() {
                                    view! {
                                        <button
                                            class="btn btn-in-cart"
                                            on:click=move |_| {
                                                let mut current_cart = cart.get();
                                                current_cart.retain(|&x| x != item_id);
                                                set_cart(current_cart);
                                            }
                                        >
                                            "Remove from Cart"
                                        </button>
                                    }
                                } else {
                                    view! {
                                        <button
                                            class="btn btn-add-cart"
                                            on:click=move |_| {
                                                let mut current_cart = cart.get();
                                                current_cart.push(item_id);
                                                set_cart(current_cart);
                                            }
                                        >
                                            "Add to Cart"
                                        </button>
                                    }
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            {move || if !cart.get().is_empty() {
                view! {
                    <div class="checkout-bar">
                        <span class="checkout-total">"Total: $"{cart_total()}</span>
                        <button class="btn btn-checkout">"Proceed to Checkout"</button>
                    </div>
                }
            } else {
                view! { <div></div> }
            }}
        </div>
    }
}