use std::{ffi::CStr, fmt::{Debug, Display}, path::Path};

use pyo3::{types::PyModule, Python};

use std::sync::Arc;


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
        PlayerAction::Invalid { path: String::new(), err_message: None }
    }
}


pub struct PlayerActionPython {
    pub name: String,
    activator: PyModule,
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
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DoesNotExists => write!(f, "Ce chemin ne mène nulle part :-("),
            ValidationError::IsNotAFile => write!(f, "Cela ne ressemble pas à un fichier..."),
            ValidationError::CannotReadFile(err_str) => write!(f, "Je n'arrive pas à lire le fichier : {}", err_str),
            ValidationError::ErrorOnLoadingCode(err_str) => write!(f, "Le code python lève une exception lors de sa lecture : {}", err_str)
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
            return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::DoesNotExists) }
        }
        if !path_obj.is_file() {
            return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::IsNotAFile) }
        }

        let file_content = match std::fs::read(path) {
            Ok(fc) => fc,
            Err(err) => {
            return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile(format!("{}", err))) };
        }};

        let file_content = match CStr::from_bytes_with_nul(&file_content) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile(format!("{}", err))) };
            }
        };

        let file_name = match path_obj.file_name() {
            Some(file_name) => file_name,
            None => {
                return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile("unable to read file name".to_owned())) };
            }
        };

        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => {
                return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile("unable to convert file name to valid UTF-8".to_owned())) };
            }
        };

        let module_name = if file_name.ends_with(".py") {
            file_name[..(file_name.len() - ".py".len())].to_owned()
        } else {
            file_name.replace(".", "_")
        };

        let file_name  = match CStr::from_bytes_with_nul(&file_name.as_bytes()) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile(format!("unable to read file name, cannot transform to cstring : {}", err))) };
            }
        };

        let module_name  = match CStr::from_bytes_with_nul(&module_name.as_bytes()) {
            Ok(cstr) => cstr,
            Err(err) => {
                return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::CannotReadFile(format!("unable to read module name, cannot transform to cstring : {}", err))) };
            }
        };

        // let (tx, rx) = std::sync::mpsc::channel();

        // Python::attach(|py| {
        //     let activators = match PyModule::from_code(py, file_content, file_name, module_name) {
        //         Ok(a) => a,
        //         Err(err) => {
        //             tx.send(PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::ErrorOnLoadingCode(format!("{}", err))) }).unwrap();
        //             return;
        //         }

        //     };
        //     tx.send(PlayerAction::Python(PlayerActionPython { name: "aaa".to_owned(), activator: activators }));
        // });

        // let python_result = match rx.recv_timeout(std::time::Duration::from_secs(2)) {
        //     Ok(pr) => pr,
        //     Err(err) => {
        //         return PlayerAction::Invalid { path: path.to_owned(), err_message: Some(ValidationError::ErrorOnLoadingCode(format!("loading timeout : {}", err))) };
        //     }
        // };

        let cache = Arc::new(PyCache::)


        PlayerAction::Invalid { path: path.to_owned(), err_message: None }
    }
}