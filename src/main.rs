pub mod services;
pub mod controllers;
pub mod interface;

use std::path::Path;
use services::pwr_service::PwrService;
use controllers::pwr_controller::PwrController;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Créer le service de gestion d'alimentation
    let pwr_service = PwrService::new();
    
    // Créer le contrôleur avec le socket et le service
    let socket_path: &Path = Path::new("/tmp/smx-iam-service");
    let controller: PwrController<'_> = PwrController::new(socket_path, pwr_service);
    
    // Démarrer le contrôleur
    controller.start().await?;
    
    Ok(())
}