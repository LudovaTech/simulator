use std::{
    collections::HashMap,
    ffi::CStr,
    fmt::{Debug, Display},
    path::Path,
};

use pyo3::{
    exceptions,
    types::{PyAnyMethods, PyModule, PyType},
    Py, Python,
};
use rapier2d::prelude::{RigidBody, RigidBodyHandle};

use crate::simulator::Simulator;

#[derive(Debug)]
pub enum PlayerAction {
    Invalid {
        path: String,
        err_message: Option<ValidationError>,
    },
    Python(PlayerActionPython),
}

impl Default for PlayerAction {
    fn default() -> Self {
        PlayerAction::Invalid {
            path: "/run/media/thomas/FS Projets/simulator/tests/test1.py".to_owned(), //String::new(), // HERE for debug purposes
            err_message: None,
        }
    }
}

pub struct PlayerActionPython {
    pub name: String,
    pub path: String,
    activator: Py<PyModule>,
}

impl Debug for PlayerActionPython {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerActionPython")
            .field("name", &self.name)
            .finish()
    }
}

#[derive(Debug)]
pub enum ValidationError {
    DoesNotExists,
    IsNotAFile,
    CannotReadFile(String),
    ErrorOnLoadingCode(String),
    TeamNameIsMissing,
    TeamNameIncorrect(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DoesNotExists => write!(f, "Ce chemin ne mène nulle part :-("),
            ValidationError::IsNotAFile => write!(f, "Cela ne ressemble pas à un fichier..."),
            ValidationError::CannotReadFile(err_str) => write!(f, "Je n'arrive pas à lire le fichier : {}", err_str),
            ValidationError::ErrorOnLoadingCode(err_str) => write!(f, "Le code python lève une exception lors de sa lecture : {}", err_str),
            ValidationError::TeamNameIsMissing => write!(f, "Il est nécessaire de spécifier le nom d'équipe dans le code. Pour python donner une variable globale `TEAM_NAME`"),
            ValidationError::TeamNameIncorrect(err_str) => write!(f, "Le nom d'équipe est illisible. Est-ce bien une string ? : {}", err_str),
        }
    }
}

impl std::error::Error for ValidationError {}

// Python code interop
impl PlayerAction {
    pub fn validate_path(path: &str) -> PlayerAction {
        if path == "" {
            return PlayerAction::default();
        }
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return PlayerAction::Invalid {
                path: path.to_owned(),
                err_message: Some(ValidationError::DoesNotExists),
            };
        }
        if !path_obj.is_file() {
            return PlayerAction::Invalid {
                path: path.to_owned(),
                err_message: Some(ValidationError::IsNotAFile),
            };
        }

        let mut file_content = match std::fs::read(path) {
            Ok(fc) => fc,
            Err(err) => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(format!("{}", err))),
                };
            }
        };

        file_content.push(b'\0');
        let file_content = match CStr::from_bytes_with_nul(&file_content) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(format!("{}", err))),
                };
            }
        };

        let file_name = match path_obj.file_name() {
            Some(file_name) => file_name,
            None => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(
                        "unable to read file name".to_owned(),
                    )),
                };
            }
        };

        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(
                        "unable to convert file name to valid UTF-8".to_owned(),
                    )),
                };
            }
        };

        let module_name = if file_name.ends_with(".py") {
            file_name[..(file_name.len() - ".py".len())].to_owned()
        } else {
            file_name.replace(".", "_")
        };

        let mut file_name_bytes = file_name.as_bytes().to_vec();
        file_name_bytes.push(b'\0');
        let file_name = match CStr::from_bytes_with_nul(&file_name_bytes) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(format!(
                        "unable to read file name, cannot transform to cstring : {}",
                        err
                    ))),
                };
            }
        };

        let mut module_name_bytes = module_name.as_bytes().to_vec();
        module_name_bytes.push(b'\0');
        let module_name = match CStr::from_bytes_with_nul(&module_name_bytes) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid {
                    path: path.to_owned(),
                    err_message: Some(ValidationError::CannotReadFile(format!(
                        "unable to read module name, cannot transform to cstring : {}",
                        err
                    ))),
                };
            }
        };

        let player_action_python: PlayerAction = Python::attach(|py| {
            // add the directory of the python main file to the python PATH.
            // This allows the main python file to import other python files that are in the same directory
            if let Some(parent) = path_obj.parent() {
                let sys = py.import("sys").unwrap();
                let pypath = sys.getattr("path").unwrap();
                pypath.call_method1("append", (parent.to_str(),)).unwrap();
            }

            let activators = match PyModule::from_code(py, file_content, file_name, module_name) {
                Ok(a) => a,
                Err(err) => {
                    return PlayerAction::Invalid {
                        path: path.to_owned(),
                        err_message: Some(ValidationError::ErrorOnLoadingCode(format!("{}", err))),
                    };
                }
            };

            // Retrieve team name now
            let pyname = match activators.getattr("TEAM_NAME") {
                Ok(name) => name,
                Err(err) => {
                    if err.is_instance_of::<exceptions::PyAttributeError>(py) {
                        return PlayerAction::Invalid {
                            path: path.to_owned(),
                            err_message: Some(ValidationError::TeamNameIsMissing),
                        };
                    } else {
                        return PlayerAction::Invalid {
                            path: path.to_owned(),
                            err_message: Some(ValidationError::TeamNameIncorrect(format!(
                                "{}",
                                err
                            ))),
                        };
                    }
                }
            };

            let name = match pyname.extract::<String>() {
                Ok(name) => name,
                Err(err) => {
                    return PlayerAction::Invalid {
                        path: path.to_owned(),
                        err_message: Some(ValidationError::TeamNameIncorrect(format!("{}", err))),
                    };
                }
            };

            // TODO : check if basic methods are there

            PlayerAction::Python(PlayerActionPython {
                name,
                path: path.to_owned(),
                activator: activators.into(),
            })
        });

        return player_action_python;
    }
}

// impl PlayerAction {
//     pub fn tick(
//         &self,
//         // We cannot borrow a mutable instance of Simulator due to borrowing restrictions
//         rigid_body_set: &mut rapier2d::prelude::RigidBodySet,
//         robot_to_rigid_body_handle: &mut HashMap<
//             crate::robot::RobotHandler,
//             RigidBodyHandle,
//         >,
//         ball_rigid_body_handle: RigidBodyHandle,
//     ) {
//         match self {
//             PlayerAction::Invalid { .. } => panic!("impossible state"),
//             PlayerAction::Python(player_action_python) => {
//                 player_action_python.tick(rigid_body_set, robot_to_rigid_body_handle, ball_rigid_body_handle)
//             }
//         }
//     }
// }

// impl PlayerActionPython {
//     pub fn tick(
//         &self,
//         // We cannot borrow a mutable instance of Simulator due to borrowing restrictions
//         self_rigid_body: &mut RigidBody,
//         robot_to_rigid_body_handle: &mut HashMap<
//             crate::robot::RobotHandler,
//             rapier2d::prelude::RigidBodyHandle,
//         >,
//         ball_rigid_body_handle: RigidBodyHandle,
//     ) {
//         Python::attach(|py| {
//             self.activator.call1(py, ()).unwrap();

//         });
//     }
// }
