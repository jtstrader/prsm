use crate::constants;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub(crate) fn write_tag_action_changes(formatted_changes: &str) -> Result<(), io::Error> {
    let path = Path::new(constants::PRSM_ROOT).join("CHANGELOG.md");
    let mut data = fs::read_to_string(&path)?;

    match data.find("## ") {
        Some(idx) => data.insert_str(idx, formatted_changes),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Could not find latest verion header '## '",
            ))
        }
    }

    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?;
    f.write_all(data.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::constants;
    use std::path::Path;

    #[test]
    fn access_changelog() {
        let path = Path::new(constants::PRSM_ROOT).join("CHANGELOG.md");
        assert!(
            path.exists(),
            "Could not locate CHANGELOG path ({})",
            path.as_os_str().to_string_lossy()
        );
    }
}
