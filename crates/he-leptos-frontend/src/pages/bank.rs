//! Bank Page - Financial transactions and account management

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub account_number: String,
    pub bank_name: String,
    pub balance: u64,
    pub is_primary: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u32,
    pub transaction_type: String,
    pub amount: u64,
    pub from: String,
    pub to: String,
    pub timestamp: String,
    pub status: String,
}

#[component]
pub fn BankPage() -> impl IntoView {
    let (accounts, set_accounts) = create_signal(vec![
        BankAccount {
            account_number: "1337-4242-6969".to_string(),
            bank_name: "CyberBank International".to_string(),
            balance: 125000,
            is_primary: true,
        },
        BankAccount {
            account_number: "9876-5432-1098".to_string(),
            bank_name: "Digital Swiss Bank".to_string(),
            balance: 500000,
            is_primary: false,
        },
    ]);

    let (transactions, set_transactions) = create_signal(vec![
        Transaction {
            id: 1,
            transaction_type: "Transfer In".to_string(),
            amount: 50000,
            from: "Anonymous".to_string(),
            to: "1337-4242-6969".to_string(),
            timestamp: "2025-09-17 14:30:00".to_string(),
            status: "Completed".to_string(),
        },
        Transaction {
            id: 2,
            transaction_type: "Hack Reward".to_string(),
            amount: 25000,
            from: "System".to_string(),
            to: "1337-4242-6969".to_string(),
            timestamp: "2025-09-17 12:15:00".to_string(),
            status: "Completed".to_string(),
        },
        Transaction {
            id: 3,
            transaction_type: "Transfer Out".to_string(),
            amount: 10000,
            from: "1337-4242-6969".to_string(),
            to: "Hardware Store".to_string(),
            timestamp: "2025-09-16 18:45:00".to_string(),
            status: "Completed".to_string(),
        },
        Transaction {
            id: 4,
            transaction_type: "Virus Income".to_string(),
            amount: 5000,
            from: "Infected-PC-42".to_string(),
            to: "9876-5432-1098".to_string(),
            timestamp: "2025-09-16 09:00:00".to_string(),
            status: "Completed".to_string(),
        },
    ]);

    let (selected_account, set_selected_account) = create_signal(0usize);
    let (transfer_modal, set_transfer_modal) = create_signal(false);
    let (transfer_amount, set_transfer_amount) = create_signal(String::new());
    let (transfer_to, set_transfer_to) = create_signal(String::new());

    let total_balance = move || {
        accounts.get().iter().map(|a| a.balance).sum::<u64>()
    };

    view! {
        <div class="bank-page">
            <div class="bank-header">
                <h1>"Bank Accounts"</h1>
                <div class="total-balance">
                    <span>"Total Balance: "</span>
                    <span class="balance-amount">"$"{total_balance()}</span>
                </div>
            </div>

            <div class="accounts-section">
                <h2>"Your Accounts"</h2>
                <div class="accounts-grid">
                    {move || accounts.get().iter().enumerate().map(|(idx, account)| {
                        let is_selected = move || selected_account.get() == idx;
                        view! {
                            <div
                                class=move || if is_selected() { "account-card selected" } else { "account-card" }
                                on:click=move |_| set_selected_account(idx)
                            >
                                <div class="account-header">
                                    <h3>{&account.bank_name}</h3>
                                    {if account.is_primary {
                                        view! { <span class="primary-badge">"PRIMARY"</span> }
                                    } else {
                                        view! { <span></span> }
                                    }}
                                </div>
                                <div class="account-number">{&account.account_number}</div>
                                <div class="account-balance">
                                    <span>"Balance: "</span>
                                    <strong>"$"{account.balance}</strong>
                                </div>
                                <div class="account-actions">
                                    <button
                                        class="btn btn-primary"
                                        on:click=move |ev| {
                                            ev.stop_propagation();
                                            set_transfer_modal(true);
                                        }
                                    >
                                        "Transfer"
                                    </button>
                                    <button class="btn btn-secondary">"Hack"</button>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="transactions-section">
                <h2>"Recent Transactions"</h2>
                <div class="transactions-table">
                    <table>
                        <thead>
                            <tr>
                                <th>"Type"</th>
                                <th>"Amount"</th>
                                <th>"From"</th>
                                <th>"To"</th>
                                <th>"Time"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || transactions.get().into_iter().map(|tx| {
                                view! {
                                    <tr class=format!("transaction-{}", tx.transaction_type.to_lowercase().replace(" ", "-"))>
                                        <td>{&tx.transaction_type}</td>
                                        <td class="amount">
                                            {if tx.transaction_type.contains("Out") {
                                                format!("-${}", tx.amount)
                                            } else {
                                                format!("+${}", tx.amount)
                                            }}
                                        </td>
                                        <td>{&tx.from}</td>
                                        <td>{&tx.to}</td>
                                        <td>{&tx.timestamp}</td>
                                        <td>
                                            <span class=format!("status status-{}", tx.status.to_lowercase())>
                                                {&tx.status}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>

            {move || if transfer_modal.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_transfer_modal(false)>
                        <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                            <h2>"Transfer Funds"</h2>
                            <form on:submit=|ev| ev.prevent_default()>
                                <div class="form-group">
                                    <label>"From Account"</label>
                                    <select class="form-control">
                                        {accounts.get().iter().map(|account| {
                                            view! {
                                                <option value=&account.account_number>
                                                    {&account.bank_name}" - $"{account.balance}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label>"To Account"</label>
                                    <input
                                        type="text"
                                        class="form-control"
                                        placeholder="Enter account number"
                                        on:input=move |ev| set_transfer_to(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-group">
                                    <label>"Amount"</label>
                                    <input
                                        type="number"
                                        class="form-control"
                                        placeholder="0"
                                        on:input=move |ev| set_transfer_amount(event_target_value(&ev))
                                    />
                                </div>
                                <div class="modal-buttons">
                                    <button
                                        type="submit"
                                        class="btn btn-success"
                                        on:click=move |_| {
                                            // Handle transfer
                                            set_transfer_modal(false);
                                        }
                                    >
                                        "Confirm Transfer"
                                    </button>
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| set_transfer_modal(false)
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            } else {
                view! { <div></div> }
            }}
        </div>
    }
}