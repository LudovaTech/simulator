use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    path::Path,
};

use nalgebra::vector;
use pyo3::{
    Py, Python, exceptions,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyModule},
};
use rerun::external::re_error::format;

#[derive(Debug)]
pub enum PlayerCode {
    Python(PlayerCodePython),
}

pub struct PlayerCodePython {
    pub name: String,
    pub path: String,
    activator: Py<PyModule>,
}

impl Debug for PlayerCodePython {
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
            PlayerCode::Python(PlayerCodePython { name, .. }) => name,
        }
    }

    #[inline]
    /// Please do this only from ui::run function !
    pub fn _set_name(&mut self, new_name: &str) {
        match self {
            PlayerCode::Python(python_code) => python_code.name = new_name.to_owned(),
        }
    }

    #[inline]
    pub fn tick(
        &self,
        player_info: PlayerInformation,
    ) -> Result<PlayerAction, CodeReturnValueError> {
        match self {
            PlayerCode::Python(python_code) => python_code.tick(player_info),
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
    UpdateFunctionIsMissing,
    UpdateFunctionIncorrect(String),
}

impl Display for CodeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeValidationError::Empty => write!(f, "Ce chemin est vide..."),
            CodeValidationError::DoesNotExists => write!(f, "Ce chemin ne mène nulle part :-("),
            CodeValidationError::IsNotAFile => write!(f, "Cela ne ressemble pas à un fichier..."),
            CodeValidationError::CannotReadFile(err_str) => {
                write!(f, "Je n'arrive pas à lire le fichier : {}", err_str)
            }
            CodeValidationError::ErrorOnLoadingCode(err_str) => write!(
                f,
                "Le code python lève une exception lors de sa lecture : {}",
                err_str
            ),
            CodeValidationError::TeamNameIsMissing => write!(
                f,
                "Il est nécessaire de spécifier le nom d'équipe dans le code. Pour python donner une variable globale `TEAM_NAME`"
            ),
            CodeValidationError::TeamNameIncorrect(err_str) => write!(
                f,
                "Le nom d'équipe est illisible. Est-ce bien une string ? : {}",
                err_str
            ),
            CodeValidationError::UpdateFunctionIsMissing => write!(
                f,
                "Le code doit contenir une fonction `update(data)` sinon je ne peux pas l'appeler."
            ),
            CodeValidationError::UpdateFunctionIncorrect(err_str) => {
                write!(f, "La fonction update est illisible : {}", err_str)
            }
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

    let mut file_content = std::fs::read(path)
        .map_err(|err| CodeValidationError::CannotReadFile(format!("{}", err)))?;

    file_content.push(b'\0');
    let file_content = CStr::from_bytes_with_nul(&file_content)
        .map_err(|err| CodeValidationError::CannotReadFile(format!("{}", err)))?;

    let file_name = path_obj.file_name().ok_or_else(|| {
        CodeValidationError::CannotReadFile("unable to read file name".to_owned())
    })?;

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

        let name = name.replace(" ", "_");

        // check if update method is here
        activators.getattr("update").map_err(|err| {
            if err.is_instance_of::<exceptions::PyAttributeError>(py) {
                CodeValidationError::UpdateFunctionIsMissing
            } else {
                CodeValidationError::UpdateFunctionIncorrect(format!("{}", err))
            }
        })?;

        Ok(PlayerCode::Python(PlayerCodePython {
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

#[derive(Debug)]
pub enum CodeReturnValueError {
    PlayerCodeException {
        code_name: String,
        err: String,
    },
    NoDict {
        code_name: String,
        err: String,
        value_returned: String,
    },
    InvalidType {
        code_name: String,
        field_name: String,
        invalid_type_hint: String,
        err: String,
        value_returned: String,
    },
    MissingField {
        code_name: String,
        missing_attribute_name: String,
        value_returned: String,
    },
    Error {
        code_name: String,
        err: String,
        value_returned: String,
    },
}

impl Display for CodeReturnValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeReturnValueError::PlayerCodeException { code_name, err } => write!(
                f,
                "Le code python de {} a levé l'exception {}",
                code_name, err
            ),
            CodeReturnValueError::Error {
                code_name,
                err,
                value_returned,
            } => write!(
                f,
                "Le code python de {}, a produit l'erreur suivante lors de la lecture du dictionnaire {}. Valeur renvoyée : {}",
                code_name, err, value_returned
            ),
            CodeReturnValueError::MissingField {
                code_name,
                missing_attribute_name,
                value_returned,
            } => write!(
                f,
                "Dans le code python de {}, l'attribut `{}` est manquant dans le dictionnaire renvoyé. Valeur renvoyée : {}",
                code_name, missing_attribute_name, value_returned
            ),
            CodeReturnValueError::InvalidType {
                code_name,
                field_name,
                invalid_type_hint,
                err,
                value_returned,
            } => write!(
                f,
                "Dans le code python de {}, la valeur renvoyée dans le champ `{}` du dictionnaire n'est pas {}. (erreur: {}) Valeur renvoyée : {}",
                code_name, field_name, invalid_type_hint, err, value_returned
            ),
            CodeReturnValueError::NoDict {
                code_name,
                err,
                value_returned,
            } => write!(
                f,
                "Dans le code python de {}, il est impossible de convertir la valeur renvoyée en un dictionnaire ({}): Valeur renvoyée : {}",
                code_name, err, value_returned
            ),
        }
    }
}

impl std::error::Error for CodeReturnValueError {}

#[derive(Debug)]
pub struct PlayerInformation {
    pub my_position: (f32, f32),
    pub friend_position: (f32, f32),
    pub enemy1_position: (f32, f32),
    pub enemy2_position: (f32, f32),
    pub ball_position: (f32, f32),
}

#[derive(Debug)]
pub struct PlayerAction {
    pub target_position: (f32, f32),
    pub power: u8,
    pub target_orientation: f32,
}

impl PlayerCodePython {
    #[inline]
    fn dict_extract<T>(
        &self,
        action: &Py<pyo3::PyAny>,
        dict: &pyo3::Bound<'_, PyDict>,
        field_name: &str,
        expected_type_hint: &str,
    ) -> Result<T, CodeReturnValueError>
    where
        for<'a, 'py> T: pyo3::FromPyObject<'a, 'py>,
        for<'a, 'py> <T as pyo3::FromPyObject<'a, 'py>>::Error: std::fmt::Display,
    {
        Ok(dict
            .get_item(field_name)
            .map_err(|err| CodeReturnValueError::Error {
                code_name: self.name.clone(),
                err: format!("{}", err),
                value_returned: format!("{}", action),
            })?
            .ok_or_else(|| CodeReturnValueError::MissingField {
                code_name: self.name.clone(),
                missing_attribute_name: field_name.to_owned(),
                value_returned: format!("{}", action),
            })?
            .extract::<T>()
            .map_err(|err| CodeReturnValueError::InvalidType {
                code_name: self.name.clone(),
                field_name: field_name.to_owned(),
                invalid_type_hint: expected_type_hint.to_owned(),
                err: format!("{}", err),
                value_returned: format!("{}", action),
            })?)
    }

    pub fn tick(
        &self,
        player_info: PlayerInformation,
    ) -> Result<PlayerAction, CodeReturnValueError> {
        Python::attach(|py| -> Result<PlayerAction, CodeReturnValueError> {
            // TODO check update existence
            let data = PyDict::new(py);
            data.set_item("my_position", player_info.my_position)
                .unwrap();
            data.set_item("friend_position", player_info.friend_position)
                .unwrap();
            data.set_item("enemy1_position", player_info.enemy1_position)
                .unwrap();
            data.set_item("enemy2_position", player_info.enemy2_position)
                .unwrap();
            data.set_item("ball_position", player_info.ball_position)
                .unwrap();
            let action = self
                .activator
                .getattr(py, "update")
                .unwrap()
                .call1(py, (data,))
                .map_err(|err| CodeReturnValueError::PlayerCodeException {
                    code_name: self.name.clone(),
                    err: format!("{}", err),
                })?;

            let dict =
                action
                    .cast_bound::<PyDict>(py)
                    .map_err(|err| CodeReturnValueError::NoDict {
                        code_name: self.name.clone(),
                        err: format!("{}", err),
                        value_returned: format!("{}", action),
                    })?;
            if dict.len() != 3 {
                println!(
                    "WARN: Le dictionnaire de retour n'a pas le nombre exact d'arguments requis"
                );
            }

            let target_position: (f32, f32) = self.dict_extract(
                &action,
                dict,
                "target_position",
                "un tuple `(float, float)`",
            )?;
            let power: u8 =
                self.dict_extract(&action, dict, "power", "un entier entre 0 et 255")?;
            let target_orientation: f32 = self.dict_extract(
                &action,
                dict,
                "target_orientation",
                "un float entre 0 et 360",
            )?;

            if !(0.0 <= target_orientation && target_orientation <= 360.0) {
                return Err(CodeReturnValueError::InvalidType {
                    code_name: self.name.clone(),
                    field_name: "target_orientation".to_owned(),
                    invalid_type_hint: "un float compris entre 0 et 360".to_owned(),
                    err: format!("c'est {}", target_orientation),
                    value_returned: format!("{}", action),
                });
            }

            Ok(PlayerAction {
                target_position,
                power,
                target_orientation,
            })
        })
    }
}
