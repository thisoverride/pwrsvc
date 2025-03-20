use std::io::{ Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use serde::{ Deserialize, Serialize};
use dbus::{blocking::Connection};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    r#type: String
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestData {
    source: String,
    version: f64,
    request: Request
}


// Structure de réponse
#[derive(Serialize, Deserialize, Debug)]
struct ResponseData {
    status: String,
    message: String,
    code: i32
}

fn main() {
    let socket = Path::new("/tmp/smx-iam-service");
    if socket.exists() {
        std::fs::remove_file(socket).unwrap();
    }
    let listener = UnixListener::bind(socket).unwrap();
    println!("Listening on socket {:?}", socket);
    println!("===> Connect using netcat: `nc -U {:?}`", socket);
    println!("===> Connect using curl: `curl -s --unix-socket {:?} http://localhost/`", socket);
    println!("===> Press Ctrl+C to exit\n");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_stream(stream);
            }
            Err(err) => {
                println!("Error while accepting connection: {:?}", err);
                break;
            }
        }
    }
}

fn handle_stream(mut stream: UnixStream) {
    let mut buf = [0; 512];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => {
            if let Ok(data) = std::str::from_utf8(&buf[..n]) {
                match serde_json::from_str::<RequestData>(data) {
                    Ok(deserialized) => {
                        println!("Reçu: {:?}", deserialized.request.r#type);

                        let response = match deserialized.request.r#type.as_str() {
                            "pwr-scsaver" => {
                                match suspend() {
                                    Ok(_) => {
                                        ResponseData {
                                            status: "success".to_string(),
                                            message: "Mise en veille effectuée".to_string(),
                                            code: 200
                                        }
                                    },
                                    Err(e) => {
                                        ResponseData {
                                            status: "error".to_string(),
                                            message: format!("Erreur lors de la mise en veille: {}", e),
                                            code: 500
                                        }
                                    }
                                }
                            },
                            "pwr-off" => {
                                match power_off() {
                                    Ok(_) => {
                                        ResponseData {
                                            status: "success".to_string(),
                                            message: "Extinction effectuée".to_string(),
                                            code: 200
                                        }
                                    },
                                    Err(e) => {
                                        ResponseData {
                                            status: "error".to_string(),
                                            message: format!("Erreur lors de l'extinction: {}", e),
                                            code: 500
                                        }
                                    }
                                }
                            },
                            "pwr-restart" => {
                                match reboot() {
                                    Ok(_) => {
                                        ResponseData {
                                            status: "success".to_string(),
                                            message: "Redémarrage effectué".to_string(),
                                            code: 200
                                        }
                                    },
                                    Err(e) => {
                                        ResponseData {
                                            status: "error".to_string(),
                                            message: format!("Erreur lors du redémarrage: {}", e),
                                            code: 500
                                        }
                                    }
                                }
                            },
                            "pwr-status" => {
                                ResponseData {
                                    status: "success".to_string(),
                                    message: "Système opérationnel".to_string(),
                                    code: 200
                                }
                            },
                            _ => {
                                println!("Type de requête inconnu: {}", deserialized.request.r#type);
                                ResponseData {
                                    status: "error".to_string(),
                                    message: format!("Type de requête inconnu: {}", deserialized.request.r#type),
                                    code: 400
                                }
                            }
                        };
                        
                        // Sérialiser la réponse en JSON
                        match serde_json::to_string(&response) {
                            Ok(json_response) => {
                                // Envoyer la réponse
                                match stream.write_all(json_response.as_bytes()) {
                                    Ok(_) => {
                                        println!("Réponse envoyée: {}", json_response);
                                    },
                                    Err(e) => {
                                        println!("Erreur lors de l'envoi de la réponse: {:?}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("Erreur lors de la sérialisation de la réponse: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Erreur de parsing JSON: {:?}", e);
                        
                        // Envoyer une réponse d'erreur
                        let error_response = ResponseData {
                            status: "error".to_string(),
                            message: format!("Format JSON invalide: {}", e),
                            code: 400
                        };
                        
                        if let Ok(json_error) = serde_json::to_string(&error_response) {
                            let _ = stream.write_all(json_error.as_bytes());
                        }
                    }
                }
            }
        },
        _ => {}
    }
}

fn power_off() -> Result<(), dbus::Error> {
    let conn: Connection = Connection::new_system()?;
    let proxy: dbus::blocking::Proxy<'_, &Connection> = conn.with_proxy("org.freedesktop.login1", "/org/freedesktop/login1", Duration::from_millis(5000));
    let _: () = proxy.method_call("org.freedesktop.login1.Manager", "PowerOff", (true,))?;
    Ok(())
}

fn reboot() -> Result<(), dbus::Error> {
    let conn: Connection = Connection::new_system()?;
    let proxy: dbus::blocking::Proxy<'_, &Connection> = conn.with_proxy("org.freedesktop.login1", "/org/freedesktop/login1", Duration::from_millis(5000));
    let _: () = proxy.method_call("org.freedesktop.login1.Manager", "Reboot", (true,))?;
    Ok(())
}

fn suspend() -> Result<(), dbus::Error> {
    let conn: Connection = Connection::new_system()?;
    let proxy: dbus::blocking::Proxy<'_, &Connection> = conn.with_proxy("org.freedesktop.login1", "/org/freedesktop/login1", Duration::from_millis(5000));
    let _: () = proxy.method_call("org.freedesktop.login1.Manager", "Suspend", (true,))?;
    Ok(())
}