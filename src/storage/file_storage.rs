use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    io::{self, BufRead},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::Error;

pub enum DataPaths {
    Log,
    Setting,
    Keys,
    SelectedKey,
}

impl DataPaths {
    pub fn get_path(&self) -> Result<PathBuf, Error> {
        let base_path = match self {
            DataPaths::Log => dirs::data_local_dir(),
            DataPaths::Setting | DataPaths::Keys | DataPaths::SelectedKey => {
                dirs::config_local_dir()
            }
        }
        .ok_or(Error::Generic(
            "Could not open well known OS directory".to_owned(),
        ))?;

        let specific_path = match self {
            DataPaths::Log => PathBuf::from("logs"),
            DataPaths::Setting => PathBuf::from("settings"),
            DataPaths::Keys => PathBuf::from("storage").join("accounts"),
            DataPaths::SelectedKey => PathBuf::from("storage").join("selected_account"),
        };

        Ok(base_path.join("notedeck").join(specific_path))
    }
}

#[derive(Debug, PartialEq)]
pub struct Directory {
    pub file_path: PathBuf,
}

impl Directory {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    /// Get the files in the current directory where the key is the file name and the value is the file contents
    pub fn get_files(&self) -> Result<HashMap<String, String>, Error> {
        let dir = fs::read_dir(self.file_path.clone())?;
        let map = dir
            .filter_map(|f| f.ok())
            .filter(|f| f.path().is_file())
            .filter_map(|f| {
                let file_name = f.file_name().into_string().ok()?;
                let contents = fs::read_to_string(f.path()).ok()?;
                Some((file_name, contents))
            })
            .collect();

        Ok(map)
    }

    pub fn get_file_names(&self) -> Result<Vec<String>, Error> {
        let dir = fs::read_dir(self.file_path.clone())?;
        let names = dir
            .filter_map(|f| f.ok())
            .filter(|f| f.path().is_file())
            .filter_map(|f| f.file_name().into_string().ok())
            .collect();

        Ok(names)
    }

    pub fn get_file(&self, file_name: String) -> Result<String, Error> {
        let filepath = self.file_path.clone().join(file_name.clone());

        if filepath.exists() && filepath.is_file() {
            let filepath_str = filepath
                .to_str()
                .ok_or_else(|| Error::Generic("Could not turn path to string".to_owned()))?;
            Ok(fs::read_to_string(filepath_str)?)
        } else {
            Err(Error::Generic(format!(
                "Requested file was not found: {}",
                file_name
            )))
        }
    }

    pub fn get_file_last_n_lines(&self, file_name: String, n: usize) -> Result<FileResult, Error> {
        let filepath = self.file_path.clone().join(file_name.clone());

        if filepath.exists() && filepath.is_file() {
            let file = File::open(&filepath)?;
            let reader = io::BufReader::new(file);

            let mut queue: VecDeque<String> = VecDeque::with_capacity(n);

            let mut total_lines_in_file = 0;
            for line in reader.lines() {
                let line = line?;

                queue.push_back(line);

                if queue.len() > n {
                    queue.pop_front();
                }
                total_lines_in_file += 1;
            }

            let output_num_lines = queue.len();
            let output = queue.into_iter().collect::<Vec<String>>().join("\n");
            Ok(FileResult {
                output,
                output_num_lines,
                total_lines_in_file,
            })
        } else {
            Err(Error::Generic(format!(
                "Requested file was not found: {}",
                file_name
            )))
        }
    }

    /// Get the file name which is most recently modified in the directory
    pub fn get_most_recent(&self) -> Result<Option<String>, Error> {
        let mut most_recent: Option<(SystemTime, String)> = None;

        for entry in fs::read_dir(&self.file_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                let modified = metadata.modified()?;
                let file_name = entry.file_name().to_string_lossy().to_string();

                match most_recent {
                    Some((last_modified, _)) if modified > last_modified => {
                        most_recent = Some((modified, file_name));
                    }
                    None => {
                        most_recent = Some((modified, file_name));
                    }
                    _ => {}
                }
            }
        }

        Ok(most_recent.map(|(_, file_name)| file_name))
    }
}

pub struct FileResult {
    pub output: String,
    pub output_num_lines: usize,
    pub total_lines_in_file: usize,
}

/// Write the file to the directory
pub fn write_file(directory: &Path, file_name: String, data: &str) -> Result<(), Error> {
    if !directory.exists() {
        fs::create_dir_all(directory)?
    }

    std::fs::write(directory.join(file_name), data)?;
    Ok(())
}

pub fn delete_file(directory: &Path, file_name: String) -> Result<(), Error> {
    let file_to_delete = directory.join(file_name.clone());
    if file_to_delete.exists() && file_to_delete.is_file() {
        fs::remove_file(file_to_delete).map_err(Error::Io)
    } else {
        Err(Error::Generic(format!(
            "Requested file to delete was not found: {}",
            file_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        storage::file_storage::{delete_file, write_file},
        Error,
    };

    use super::Directory;

    static CREATE_TMP_DIR: fn() -> Result<PathBuf, Error> =
        || Ok(tempfile::TempDir::new()?.path().to_path_buf());

    #[test]
    fn test_add_get_delete() {
        if let Ok(path) = CREATE_TMP_DIR() {
            let directory = Directory::new(path);
            let file_name = "file_test_name.txt".to_string();
            let file_contents = "test";
            let write_res = write_file(&directory.file_path, file_name.clone(), file_contents);
            assert!(write_res.is_ok());

            if let Ok(asserted_file_contents) = directory.get_file(file_name.clone()) {
                assert_eq!(asserted_file_contents, file_contents);
            } else {
                panic!("File not found");
            }

            let delete_res = delete_file(&directory.file_path, file_name);
            assert!(delete_res.is_ok());
        } else {
            panic!("could not get interactor")
        }
    }

    #[test]
    fn test_get_multiple() {
        if let Ok(path) = CREATE_TMP_DIR() {
            let directory = Directory::new(path);

            for i in 0..10 {
                let file_name = format!("file{}.txt", i);
                let write_res = write_file(&directory.file_path, file_name, "test");
                assert!(write_res.is_ok());
            }

            if let Ok(files) = directory.get_files() {
                for i in 0..10 {
                    let file_name = format!("file{}.txt", i);
                    assert!(files.contains_key(&file_name));
                    assert_eq!(files.get(&file_name).unwrap(), "test");
                }
            } else {
                panic!("Files not found");
            }

            if let Ok(file_names) = directory.get_file_names() {
                for i in 0..10 {
                    let file_name = format!("file{}.txt", i);
                    assert!(file_names.contains(&file_name));
                }
            } else {
                panic!("File names not found");
            }

            for i in 0..10 {
                let file_name = format!("file{}.txt", i);
                assert!(delete_file(&directory.file_path, file_name).is_ok());
            }
        } else {
            panic!("could not get interactor")
        }
    }
}