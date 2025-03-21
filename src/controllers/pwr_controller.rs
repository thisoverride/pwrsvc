use std::path::Path;
use crate::services::PwrService;
use crate::interface::model::{RequestData, ResponseData};
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use futures::stream::StreamExt;

// Structure PwrController qui gère le socket et les communications
pub struct PwrController<'a> {
    socket: &'a Path,
    service: PwrService
}

impl<'a> PwrController<'a> {
    // Constructeur
    pub fn new(socket_path: &'a Path, pwr_service: PwrService) -> Self {
        PwrController {
            socket: socket_path,
            service: pwr_service
        }
    }

    // Méthode pour démarrer le contrôleur
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.socket.exists() {
            tokio::fs::remove_file(self.socket).await?;
        }
        
        let listener = UnixListener::bind(self.socket)?;
        
        println!("Listening on socket {:?}", self.socket);
        println!("===> Connect using netcat: `nc -U {:?}`", self.socket);
        println!("===> Connect using curl: `curl -s --unix-socket {:?} http://localhost/`", self.socket);
        println!("===> Press Ctrl+C to exit\n");
        
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    // Clone le service pour chaque connexion
                    let service = self.service.clone();
                    
                    // Gérer chaque connexion dans une tâche séparée
                    tokio::spawn(async move {
                        if let Err(e) = handle_stream(stream, service).await {
                            eprintln!("Error handling stream: {}", e);
                        }
                    });
                }
                Err(err) => {
                    println!("Error while accepting connection: {:?}", err);
                    break;
                }
            }
        }
        
        Ok(())
    }
}

// Fonction pour gérer une connexion individuelle
async fn handle_stream(mut stream: UnixStream, service: PwrService) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0; 512];
    
    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => {
            if let Ok(data) = std::str::from_utf8(&buf[..n]) {
                match serde_json::from_str::<RequestData>(data) {
                    Ok(deserialized) => {
                        println!("Reçu: {:?}", deserialized.request.r#type);
                        
                        // Utiliser le PwrService pour traiter la commande
                        let response = service.handle_command(&deserialized.request.r#type).await;
                        
                        // Sérialiser la réponse en JSON
                        let json_response = serde_json::to_string(&response)?;
                        
                        // Envoyer la réponse
                        stream.write_all(json_response.as_bytes()).await?;
                        println!("Réponse envoyée: {}", json_response);
                    },
                    Err(e) => {
                        println!("Erreur de parsing JSON: {:?}", e);
                        // Envoyer une réponse d'erreur
                        let error_response = ResponseData {
                            status: "error".to_string(),
                            message: format!("Format JSON invalide: {}", e),
                            code: 400
                        };
                        let json_error = serde_json::to_string(&error_response)?;
                        stream.write_all(json_error.as_bytes()).await?;
                    }
                }
            }
        },
        _ => {}
    }
    
    Ok(())
}