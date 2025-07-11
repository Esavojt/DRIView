use std::{
    collections::{HashMap, hash_map::Entry},
    fs,
    io::{self, ErrorKind},
};

// Paths to the pci.ids file
const PCIID_PATHS: [&str; 3] = [
    "/usr/share/hwdata/pci.ids", // Arch
    "/usr/share/misc/pci.ids", // Debian
    "pci.ids", // Local file
];

#[derive(Debug)]
pub struct Device {
    device_name: Option<String>,
    device_id: String,
    vendor_name: Option<String>,
    vendor_id: String,
    drm_path: Vec<String>,
}

impl Device {
    pub fn new(vendor: &str, device: &str, drm_path: String) -> io::Result<Self> {
        Self::get_device_from_pciid(vendor, device, drm_path)
    }

    fn get_device_from_pciid(vendor: &str, device: &str, drm_path: String) -> io::Result<Self> {
        // Get pci.ids file content from a valid file
        let mut pci_ids_content = String::new();
        for path in PCIID_PATHS {
            match fs::read_to_string(path) {
                Ok(cont) => {
                    pci_ids_content = cont;
                    break;
                }
                Err(err) => {
                    if err.kind() != ErrorKind::NotFound {
                        return Err(err);
                    }
                }
            }
        }

        if pci_ids_content.is_empty() {
            println!(
                "WARNING: pci.ids file not found, can't translate device id to vendor information"
            );
        }
        //  Create empty Device
        let mut dev = Self {
            device_id: device.to_owned(),
            device_name: None,
            vendor_id: vendor.to_owned(),
            vendor_name: None,
            drm_path: vec![drm_path],
        };

        let mut lines = pci_ids_content.lines();

        // Convert device id and vendor id to names
        let vendor_line = lines.find(|line| line.starts_with(vendor));

        if let Some(vendor_line) = vendor_line {
            let vendor_name = vendor_line.get(4..).unwrap_or("").trim().to_owned();

            dev.vendor_name = Some(vendor_name);

            // Find device name
            for line in lines {
                if line.starts_with(&format!("\t{}", device)) { // Found device id
                    if let Some(device_name) = line.get(5..) {
                        dev.device_name = Some(device_name.trim().to_owned());
                        return Ok(dev);
                    }
                } else if line.starts_with("#") || line.starts_with("\t") { // Reached comment or not our device id
                    continue;
                } else { // Reached the end of the device list
                    return Ok(dev);
                }
            }
        }
        Ok(dev)
    }

    pub fn get_device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
    pub fn get_vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }
    pub fn get_vendor_id(&self) -> &str {
        &self.vendor_id
    }
    pub fn get_drm_path(&self) -> &[String] {
        &self.drm_path
    }

    pub fn contains_path(&self, path: &String) -> bool {
        self.drm_path.contains(path)
    }

    pub fn vendor_name_pretty(&self) -> &str {
        self.vendor_name
            .as_deref()
            .unwrap_or("Unknown Manufacturer")
    }
    pub fn device_name_pretty(&self) -> &str {
        self.device_name
            .as_deref()
            .unwrap_or("Unknown Device")
    }
}

pub fn get_gpus() -> io::Result<HashMap<String, Device>> {
    // PCIID : Device
    let mut gpus: HashMap<String, Device> = HashMap::new();

    // Get all dri devices in /dev/dri
    // drwxr-xr-x   3 root root        100 čec 11 10:42 .
    // drwxr-xr-x  21 root root       3820 čec 11 10:42 ..
    // drwxr-xr-x   2 root root         80 čec 11 10:42 by-path
    // crw-rw----+  1 root video  226,   0 čec 11 11:20 card0
    // crw-rw----+  1 root render 226, 128 čec 11 10:42 renderD128

    for entry in fs::read_dir("/dev/dri")? {
        let entry = entry?;
        
        // Check if it is not a dir (ignore the by-path dir)
        if !entry.metadata()?.is_dir() {
            // Get the vendor and device id from sysfs
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let vendor = fs::read_to_string(format!("/sys/class/drm/{}/device/vendor", file_name))?;
            let vendor = vendor.trim();

            let device = fs::read_to_string(format!("/sys/class/drm/{}/device/device", file_name))?;
            let device = device.trim();
            println!("Found GPU: {file_name} - {vendor}:{device}");

            // Get the pci path so we can link card and renderD together as one GPU card
            let pci_path = fs::read_link(format!("/sys/class/drm/{}/device", file_name))?;
            let pci_path = pci_path.as_path().to_string_lossy();
            let drm_path = format!("/dev/dri/{}", file_name);

            // If the path already exists, append the drm path, else create new Device entry
            match gpus.entry(pci_path.to_string()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().drm_path.push(drm_path);
                }
                Entry::Vacant(entry) => {
                    entry.insert(Device::new(
                        vendor.strip_prefix("0x").unwrap_or(vendor),
                        device.strip_prefix("0x").unwrap_or(device),
                        drm_path,
                    )?);
                }
            }
        }
    }
    Ok(gpus)
}
