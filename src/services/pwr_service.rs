use dbus::blocking::Connection;
use std::time::Duration;
use crate::interface::model::ResponseData;
use std::fmt;

// Définir un type d'erreur personnalisé pour plus de clarté
#[derive(Debug)]
pub enum PowerError {
    DBusError(dbus::Error),
    ExecutionError(String),
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerError::DBusError(e) => write!(f, "Erreur DBus: {}", e),
            PowerError::ExecutionError(s) => write!(f, "Erreur d'exécution: {}", s),
        }
    }
}

impl From<dbus::Error> for PowerError {
    fn from(error: dbus::Error) -> Self {
        PowerError::DBusError(error)
    }
}

impl std::error::Error for PowerError {}

// Définir un trait pour les opérations de gestion d'alimentation
pub trait PowerManagement {
    async fn power_off(&self) -> Result<(), PowerError>;
    async fn reboot(&self) -> Result<(), PowerError>;
    async fn suspend(&self) -> Result<(), PowerError>;
    async fn get_power_status(&self) -> Result<String, PowerError>;
}

// Configuration pour les services D-Bus
#[derive(Clone)]
pub struct DBusConfig {
    pub service: String,
    pub path: String,
    pub interface: String,
    pub timeout_ms: u64,
}

impl Default for DBusConfig {
    fn default() -> Self {
        Self {
            service: "org.freedesktop.login1".to_string(),
            path: "/org/freedesktop/login1".to_string(),
            interface: "org.freedesktop.login1.Manager".to_string(),
            timeout_ms: 5000,
        }
    }
}

// Le service de gestion d'alimentation
#[derive(Clone)]
pub struct PwrService {
    dbus_config: DBusConfig,
}

impl PwrService {
    // Constructeur
    pub fn new() -> Self {
        PwrService {
            dbus_config: DBusConfig::default(),
        }
    }

    // Constructeur avec configuration personnalisée
    pub fn with_config(config: DBusConfig) -> Self {
        PwrService {
            dbus_config: config,
        }
    }

    // Méthode générique pour exécuter une commande D-Bus
    async fn execute_dbus_command(&self, method: &str, params: (bool,)) -> Result<(), PowerError> {
        let config = self.dbus_config.clone();
        
        tokio::task::spawn_blocking(move || -> Result<(), PowerError> {
            let conn = Connection::new_system()?;
            let proxy = conn.with_proxy(
                &config.service, 
                &config.path, 
                Duration::from_millis(config.timeout_ms)
            );
            
            proxy.method_call(&config.interface, method, params)?;
            Ok(())
        })
        .await
        .map_err(|e| PowerError::ExecutionError(format!("Erreur de tâche: {}", e)))?
    }

    // Méthode pour gérer les commandes d'alimentation
    pub async fn handle_command(&self, command: &str) -> ResponseData {
        match command {
            "pwr-scsaver" => self.handle_suspend_command().await,
            "pwr-off" => self.handle_poweroff_command().await,
            "pwr-restart" => self.handle_reboot_command().await,
            "pwr-status" => self.handle_status_command().await,
            _ => {
                println!("Type de requête inconnu: {}", command);
                ResponseData {
                    status: "error".to_string(),
                    message: format!("Type de requête inconnu: {}", command),
                    code: 400
                }
            }
        }
    }

    async fn handle_suspend_command(&self) -> ResponseData {
        match self.suspend().await {
            Ok(_) => ResponseData {
                status: "success".to_string(),
                message: "Mise en veille effectuée".to_string(),
                code: 200
            },
            Err(e) => ResponseData {
                status: "error".to_string(),
                message: format!("Erreur lors de la mise en veille: {}", e),
                code: 500
            }
        }
    }

    async fn handle_poweroff_command(&self) -> ResponseData {
        match self.power_off().await {
            Ok(_) => ResponseData {
                status: "success".to_string(),
                message: "Extinction effectuée".to_string(),
                code: 200
            },
            Err(e) => ResponseData {
                status: "error".to_string(),
                message: format!("Erreur lors de l'extinction: {}", e),
                code: 500
            }
        }
    }

    async fn handle_reboot_command(&self) -> ResponseData {
        match self.reboot().await {
            Ok(_) => ResponseData {
                status: "success".to_string(),
                message: "Redémarrage effectué".to_string(),
                code: 200
            },
            Err(e) => ResponseData {
                status: "error".to_string(),
                message: format!("Erreur lors du redémarrage: {}", e),
                code: 500
            }
        }
    }

    async fn handle_status_command(&self) -> ResponseData {
        match self.get_power_status().await {
            Ok(status) => ResponseData {
                status: "success".to_string(),
                message: status,
                code: 200
            },
            Err(e) => ResponseData {
                status: "error".to_string(),
                message: format!("Erreur lors de la vérification du statut: {}", e),
                code: 500
            }
        }
    }
}

// Implémentation du trait PowerManagement pour PwrService
impl PowerManagement for PwrService {
    async fn power_off(&self) -> Result<(), PowerError> {
        self.execute_dbus_command("PowerOff", (true,)).await
    }

    async fn reboot(&self) -> Result<(), PowerError> {
        self.execute_dbus_command("Reboot", (true,)).await
    }

    async fn suspend(&self) -> Result<(), PowerError> {
        self.execute_dbus_command("Suspend", (true,)).await
    }

    async fn get_power_status(&self) -> Result<String, PowerError> {
        // Dans un cas réel, vous pourriez vérifier l'état du système
        // via D-Bus ou d'autres moyens
        Ok("Système opérationnel".to_string())
    }
}

