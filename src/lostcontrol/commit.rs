use serde::{self, Deserialize, Serialize};

const TIME_FORMAT_STRING: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub id: usize,
    pub message: String,
    pub creation_datetime: String,
    pub modified_files: Vec<String>,
}

impl Commit {
    pub fn new(id: usize, message: String, modified_files: Vec<String>) -> Commit {
        return Commit {
            id: id, 
            message: message,
            creation_datetime: chrono::Utc::now().to_rfc3339(),
            modified_files: modified_files,
        }
    }

    pub fn get_time_formatted(&self) -> String {
        let creation_datetime: chrono::DateTime<chrono::Local> = 
            chrono::DateTime::from(chrono::DateTime::parse_from_rfc3339(&self.creation_datetime).unwrap());
        return creation_datetime.format(TIME_FORMAT_STRING).to_string();
    }
}

impl std::fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ID: {}\nMessage: {}\nCreated at: {}\nModified Files:\n", 
            self.id,
            self.message,
            self.get_time_formatted(),
        )?;
        for file in &self.modified_files {
            write!(f, "  {}\n", file)?;
        }
        return Ok(());
    }
}

// simple clone trait
impl Clone for Commit {
    fn clone(&self) -> Commit {
        return Commit {
            id: self.id,
            message: self.message.clone(),
            creation_datetime: self.creation_datetime.clone(),
            modified_files: self.modified_files.clone(),
        }
    }
}