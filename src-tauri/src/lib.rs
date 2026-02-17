use log::info;
use tauri::Manager;

mod commands;
mod database;
mod mock_api;
mod models;
mod print_invoice;
mod print_receipt;
mod print_daily_report;
mod api_client;

#[cfg(not(debug_assertions))]
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app".into()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            info!("App started, version: {}", app.package_info().version);

            // Initialize database
            let db = database::init_database(app.handle())
                .expect("Failed to initialize database");
            app.manage(db);

            #[cfg(not(debug_assertions))]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    updater::check_and_install_updates(handle).await;
                });
            }

            #[cfg(debug_assertions)]
            info!("Dev mode - skipping auto-updater");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Sync commands
            commands::clear_database,
            commands::delete_partners_and_locations,
            commands::check_first_run,
            commands::get_sync_status,
            commands::sync_all_data,
            commands::check_online_status,
            // Partner commands
            commands::get_partners,
            commands::search_partners,
            // Product commands
            commands::get_products,
            commands::search_products,
            // Invoice commands
            commands::create_invoice,
            commands::get_invoices,
            commands::get_invoice_detail,
            commands::send_invoice,
            commands::preview_invoice_json,
            commands::send_all_pending_invoices,
            commands::cancel_invoice_sending,
            commands::delete_invoice,
            commands::print_invoice_to_html,
            commands::preview_invoice_certificate,
            commands::print_collection_to_html,
            commands::get_available_printers,
            // Agent settings commands
            commands::get_agent_settings,
            commands::save_agent_settings,
            // Collection & Balance commands
            commands::sync_client_balances,
            commands::get_client_balances,
            commands::record_collection,
            commands::record_collection_group,
            commands::record_collection_from_invoice,
            commands::get_collections,
            commands::sync_collections,
            commands::send_collection,
            commands::delete_collection,
            commands::get_sales_report,
            commands::get_sales_print_report,
            commands::get_sales_products_report,
            commands::get_collections_report,
            commands::print_daily_report,
            commands::print_report_html,
            // API test commands
            commands::test_api_partners,
            commands::test_api_articles,
            // Debug commands
            commands::debug_db_counts,
            commands::debug_partner_payment_terms,
            commands::update_all_partners_payment_terms,
            commands::save_report_html,
            commands::open_external_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
