// Déclarer les modules
pub mod services;
pub mod controllers;
pub mod interface;

use std::path::Path;
use services::pwr_service::PwrService;
use controllers::pwr_controller::PwrController;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pwr_service = PwrService::new();
    let socket_path = Path::new("/run/pwrsvc.sock");
    let controller = PwrController::new(socket_path, pwr_service);
    controller.start().await?;
    
    Ok(())
}