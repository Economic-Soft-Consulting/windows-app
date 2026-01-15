use log::info;
use tauri::Manager;

mod commands;
mod database;
mod mock_api;
mod models;
mod print_invoice;

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
            commands::delete_invoice,
            commands::print_invoice_to_html,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
