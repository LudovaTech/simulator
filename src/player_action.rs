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
pub enum PlayerCode {
    Python(PlayerActionPython),
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

impl PlayerCode {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            PlayerCode::Python(PlayerActionPython { name, .. }) => name
        }
    }

    #[inline]
    /// Please do this only from ui::run function !
    pub fn _set_name(&mut self, new_name: &str) {
        match self {
            PlayerCode::Python(python_code) => python_code.name = new_name.to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum CodeValidationError {
    Empty,
    DoesNotExists,
    IsNotAFile,
    CannotReadFile(String),
    ErrorOnLoadingCode(String),
    TeamNameIsMissing,
    TeamNameIncorrect(String),
}

impl Display for CodeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeValidationError::Empty => write!(f, "Ce chemin est vide..."),
            CodeValidationError::DoesNotExists => write!(f, "Ce chemin ne mène nulle part :-("),
            CodeValidationError::IsNotAFile => write!(f, "Cela ne ressemble pas à un fichier..."),
            CodeValidationError::CannotReadFile(err_str) => write!(f, "Je n'arrive pas à lire le fichier : {}", err_str),
            CodeValidationError::ErrorOnLoadingCode(err_str) => write!(f, "Le code python lève une exception lors de sa lecture : {}", err_str),
            CodeValidationError::TeamNameIsMissing => write!(f, "Il est nécessaire de spécifier le nom d'équipe dans le code. Pour python donner une variable globale `TEAM_NAME`"),
            CodeValidationError::TeamNameIncorrect(err_str) => write!(f, "Le nom d'équipe est illisible. Est-ce bien une string ? : {}", err_str),
        }
    }
}

impl std::error::Error for CodeValidationError {}

pub fn validate_path(path: &str) -> Result<PlayerCode, CodeValidationError> {
    if path == "" {
        return Err(CodeValidationError::Empty);
    }
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(CodeValidationError::DoesNotExists);
    }
    if !path_obj.is_file() {
        return Err(CodeValidationError::IsNotAFile);
    }

    let mut file_content =
        std::fs::read(path).map_err(|err| CodeValidationError::CannotReadFile(format!("{}", err)))?;

    file_content.push(b'\0');
    let file_content = CStr::from_bytes_with_nul(&file_content)
        .map_err(|err| CodeValidationError::CannotReadFile(format!("{}", err)))?;

    let file_name = path_obj
        .file_name()
        .ok_or_else(|| CodeValidationError::CannotReadFile("unable to read file name".to_owned()))?;

    let file_name = file_name.to_str().ok_or_else(|| {
        CodeValidationError::CannotReadFile("unable to convert file name to valid UTF-8".to_owned())
    })?;

    let module_name = if file_name.ends_with(".py") {
        file_name[..(file_name.len() - ".py".len())].to_owned()
    } else {
        file_name.replace(".", "_")
    };

    let mut file_name_bytes = file_name.as_bytes().to_vec();
    file_name_bytes.push(b'\0');
    let file_name = CStr::from_bytes_with_nul(&file_name_bytes).map_err(|err| {
        CodeValidationError::CannotReadFile(format!(
            "unable to read file name, cannot transform to cstring : {}",
            err
        ))
    })?;

    let mut module_name_bytes = module_name.as_bytes().to_vec();
    module_name_bytes.push(b'\0');
    let module_name = CStr::from_bytes_with_nul(&module_name_bytes).map_err(|err| {
        CodeValidationError::CannotReadFile(format!(
            "unable to read module name, cannot transform to cstring : {}",
            err
        ))
    })?;

    let player_action_python: Result<PlayerCode, CodeValidationError> = Python::attach(|py| {
        // add the directory of the python main file to the python PATH.
        // This allows the main python file to import other python files that are in the same directory
        if let Some(parent) = path_obj.parent() {
            let sys = py.import("sys").unwrap();
            let pypath = sys.getattr("path").unwrap();
            pypath.call_method1("append", (parent.to_str(),)).unwrap();
        }

        let activators = PyModule::from_code(py, file_content, file_name, module_name)
            .map_err(|err| CodeValidationError::ErrorOnLoadingCode(format!("{}", err)))?;

        // Retrieve team name now
        let pyname = activators.getattr("TEAM_NAME").map_err(|err| {
            if err.is_instance_of::<exceptions::PyAttributeError>(py) {
                CodeValidationError::TeamNameIsMissing
            } else {
                CodeValidationError::TeamNameIncorrect(format!("{}", err))
            }
        })?;

        let name = pyname
            .extract::<String>()
            .map_err(|err| CodeValidationError::TeamNameIncorrect(format!("{}", err)))?;

        // TODO : check if basic methods are there

        Ok(PlayerCode::Python(PlayerActionPython {
            name,
            path: path.to_owned(),
            activator: activators.into(),
        }))
    });

    return player_action_python;
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
