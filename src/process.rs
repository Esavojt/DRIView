use std::{
    fs,
    io::{self, ErrorKind},
};

pub fn get_processes() -> io::Result<Vec<Process>> {
    let mut pids: Vec<String> = Vec::new();
    // Get running processes from procfs
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Check if the directory name is numeric (a PID)
        if file_name_str.chars().all(|c| c.is_ascii_digit()) {
            pids.push(file_name_str.into_owned());
            //println!("{}", file_name_str);
        }
    }

    let mut processes: Vec<Process> = Vec::with_capacity(pids.len());
    // Create new process, if we can't read the file descriptor, skip it
    for pid in pids {
        match Process::new(pid) {
            Ok(proc) => {
                processes.push(proc);
            }
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied | ErrorKind::NotFound => continue,
                _ => return Err(error),
            },
        }
    }

    Ok(processes)
}

#[derive(Debug)]
pub struct Process {
    pid: String,
    fds: Vec<String>,
    name: String,
}

impl Process {
    pub fn new(pid: String) -> io::Result<Self> {
        let mut fds = Vec::new();
        // Get all open fd
        let dir = fs::read_dir(format!("/proc/{}/fd", pid))?;

        for fd in dir {
            match fs::read_link(fd?.path()) {
                Ok(link) => {
                    let path_name = link.to_string_lossy();
                    fds.push(path_name.to_string());
                }
                Err(_) => continue,
            }
        }
        // Get the process name
        let mut name = String::new();
        let proc_status = fs::read_to_string(format!("/proc/{}/status", pid))?;

        if let Some(line) = proc_status.lines().find(|line| line.starts_with("Name")) {
            name = line[5..].trim().to_owned();
        }

        Ok(Self { pid, fds, name })
    }
    pub fn get_pid(&self) -> &str {
        &self.pid
    }
    pub fn get_fds(&self) -> &[String] {
        &self.fds
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
