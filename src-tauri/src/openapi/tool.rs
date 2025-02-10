pub mod str_tool {
    use regex::Regex;

    pub fn capture(re: &Regex, target: &str) -> Option<String> {
        if let Some(captures) = re.captures(target) {
            return Some(captures.get(1).unwrap().as_str().to_string());
        }
        None
    }
}

pub mod time_tool {
    use std::time::SystemTime;

    pub fn get_r() -> (String, String) {
        let local_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        (
            (-1 * local_time.as_secs() as i64 / 1579).to_string(),
            local_time.as_secs().to_string(),
        )
    }

    pub fn get_msg_id() -> String {
        let local_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        local_time.as_micros().to_string()
    }
}

pub mod file_tool {
    use std::path::{Path};
    use std::{fs, io};

    pub fn get_or_create_file(path: &Path) -> io::Result<String> {
        if path.exists() {
            return fs::read_to_string(path);
        }
        fs::File::create(path)?;
        Ok("".to_string())
    }
}
