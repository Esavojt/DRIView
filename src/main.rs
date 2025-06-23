use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let gpus = get_gpus()?;

    let mut pids: Vec<String> = Vec::new();
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

    for pid in pids {
        match Process::new(pid) {
            Ok(proc) => {
                processes.push(proc);
            },
            Err(error) => {
                match error.kind() {
                    ErrorKind::PermissionDenied | ErrorKind::NotFound => continue,
                    _ => return Err(error),
                }
            }
        }
    }

    let mut output: Vec<GPUProcesses> = gpus.iter()
        .map(|(path,dev)| GPUProcesses { device_path: path, device: dev, processes: Vec::new()})
        .collect();

    for proc in &processes {
        for fd in &proc.fds {
            if let Some(gpu) = output.iter_mut().find(|gpu| *gpu.device_path == *fd) {
                gpu.processes.push(proc);
            }
        }
    }
    

    for out in &output {
        println!("GPU {} {} ({}:{}) [{}] is used by:",
            out.device.vendor_name.as_ref().unwrap_or(&"Unknown Manufacturer".to_owned()),
            out.device.device_name.as_ref().unwrap_or(&"Unknown Device".to_owned()),
            out.device.vendor_id,
            out.device.device_id,
            out.device_path
        );

        for proc in &out.processes {
            println!("({}) {}", proc.pid, proc.name);
        }
    }

    //println!("{:?}", get_name_by_pciid("1002", "67df")?);
    //println!("{:?}", get_gpus()?);
    Ok(())
}


fn get_gpus() -> io::Result<HashMap<String, Device>> {
    let mut gpus: HashMap<String, Device> = HashMap::new();
    for entry in fs::read_dir("/dev/dri")? {
        let entry = entry?;
        if !entry.metadata()?.is_dir() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let vendor = fs::read_to_string(format!("/sys/class/drm/{}/device/vendor", file_name))?;
            let vendor = vendor.trim();

            let device = fs::read_to_string(format!("/sys/class/drm/{}/device/device", file_name))?;
            let device = device.trim();
            println!("Found GPU: {file_name} - {vendor}:{device}");
            let dev = Device::new(vendor.strip_prefix("0x").unwrap_or(vendor), device.strip_prefix("0x").unwrap_or(device))?;

            gpus.insert(format!("/dev/dri/{}", file_name), dev);
        }
    }
    Ok(gpus)
}

#[derive(Debug)]
struct Device {
    device_name: Option<String>,
    device_id: String,
    vendor_name: Option<String>,
    vendor_id: String,
}
#[derive(Debug)]
struct Process {
    pid: String,
    fds: Vec<String>,
    name: String
}

#[derive(Debug)]
struct GPUProcesses<'a> {
    device_path: &'a String,
    device: &'a Device,
    processes: Vec<&'a Process>
}

impl Device {
    pub fn new(vendor: &str, device: &str) -> io::Result<Self> {
        Self::get_device_from_pciid(vendor, device)
    }

    fn get_device_from_pciid(vendor: &str, device: &str) -> io::Result<Self> {
        let pci_ids_content = fs::read_to_string("pci.ids")?;

        let mut dev = Self {
            device_id: device.to_owned(),
            device_name: None,
            vendor_id: vendor.to_owned(),
            vendor_name: None
        };

        let mut lines = pci_ids_content.lines();

        let vendor_line = lines
            .find(|line| line.starts_with(vendor));

        if let Some(vendor_line) = vendor_line {
            let vendor_name = vendor_line.get(4..).unwrap_or("").trim().to_owned();

            dev.vendor_name = Some(vendor_name);

            for line in lines {
                if line.starts_with(&format!("\t{}", device)) {
                if let Some(device_name) = line.get(5..) {
                    dev.device_name = Some(device_name.trim().to_owned());
                    return Ok(dev)
                }
                } else if line.starts_with("#") || line.starts_with("\t") {
                    continue;
                } else {
                    return Ok(dev)
                }
            }
        }
        Ok(dev)
    }
}

impl Process {
    fn new(pid: String) -> io::Result<Self> {
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

        let mut name = String::new();
        let proc_status = fs::read_to_string(format!("/proc/{}/status", pid))?;

        if let Some(line) = proc_status.lines().find(|line| line.starts_with("Name")) {
            name = line[5..].trim().to_owned();
        }

        Ok(Self {
            pid,
            fds,
            name
        })
    }
}