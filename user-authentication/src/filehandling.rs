use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// creates a file named credentials.txt in current directory
///
///
/// # Errors
///
/// `io::Error`
pub fn create_file() -> std::io::Result<()> {
    let _file = File::create("credentials.txt")?;
    Ok(())
}

/// write user's credentials to credentials.txt
///
/// # Arguments
///
/// * `password` - &str
/// * 'hashpassword' - PasswordHash<> => argon PHC string format
///
/// # Returns
///
/// Returns a `Result` containing () if succesful, or an `io::Error` if an error occurs
///
/// # Errors
///
///  `io::Error`
pub fn write_to_file(_username: &str, _hashpassword: &str) -> std::io::Result<()> {
    let formatted_string = format!("username: {:?}, password: {:?}", _username, _hashpassword);

    fs::write("credentials.txt", formatted_string)?;
    Ok(())
}

/// find password based on username in credentials.txt
///
/// # Arguments
///
/// * `username` - &str
///
/// # Returns
///
/// Returns a `Result` containing Option<> with password if succesful, or an `io::Error` if an error occurs
///
/// # Errors
///
///  `io::Error`
pub fn find_password(username: &str) -> io::Result<Option<String>> {
    let file = File::open("credentials.txt")?;
    let _reader = BufReader::new(file);

    let username_password_re = Regex::new(r#"username: "(.*?)", password: "(.*?)""#).unwrap();

    for line in _reader.lines() {
        let line = line?;
        let mut found_password = "";
        if let Some(captures) = username_password_re.captures(&line) {
            found_password = captures.get(2).map_or("", |m| m.as_str()).trim_matches('"');
        }
        let valid = match find_username(username) {
            Ok(Some(username)) => username,
            Ok(None) => panic!("No password found"),
            Err(_) => panic!("some other error"),
        };
        if valid {
            return Ok(Some(found_password.to_string()));
        }
    }
    Ok(None)
}

/// find username in credentials.txt
///
/// # Arguments
///
/// * `username` - &str
///
/// # Returns
///
/// Returns a `Result` containing Option<> with true if succesful, or an `Option<> with false` if no username found
///
pub fn find_username(username: &str) -> io::Result<Option<bool>> {
    let file = File::open("credentials.txt")?;
    let _reader = BufReader::new(file);

    let username_password_re = Regex::new(r#"username: "(.*?)", password: "(.*?)""#).unwrap();

    for line in _reader.lines() {
        let line = line?;
        let mut found_username = "";
        if let Some(captures) = username_password_re.captures(&line) {
            found_username = captures.get(1).map_or("", |m| m.as_str()).trim_matches('"');
        }
        if found_username == username {
            return Ok(Some(true));
        }
    }
    Ok(Some(false))
}
