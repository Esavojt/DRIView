use std::collections::HashMap;
use std::io;

use crate::process::Process;

mod gpu;
mod process;

fn main() -> io::Result<()> {
    let gpus = gpu::get_gpus()?;

    let processes = process::get_processes()?;

    let gpu_procs = link_processes_to_gpu(&processes, &gpus);

    for out in &gpu_procs {
        println!(
            "\n{} {}\n({}:{}) [{}] is used by:",
            out.device.vendor_name_pretty(),
            out.device.device_name_pretty(),
            out.device.get_vendor_id(),
            out.device.get_device_id(),
            out.device_path.replace("../", "")
        );

        for proc in &out.processes {
            println!("({}) {}", proc.get_pid(), proc.get_name());
        }
    }

    //println!("{:?}", get_name_by_pciid("1002", "67df")?);
    //println!("{:?}", get_gpus()?);
    Ok(())
}

fn link_processes_to_gpu<'a>(
    procs: &'a [Process],
    gpus: &'a HashMap<String, gpu::Device>,
) -> Vec<GPUProcessInfo<'a>> {
    let mut gpu_procs: Vec<GPUProcessInfo> = gpus
        .iter()
        .map(|(path, dev)| GPUProcessInfo {
            device_path: path,
            device: dev,
            processes: Vec::new(),
        })
        .collect();

    for proc in procs.iter() {
        for fd in proc.get_fds() {
            for gpu in gpu_procs.iter_mut() {
                if gpu.device.contains_path(fd) {
                    gpu.processes.push(proc);
                    break;
                }
            }
        }
    }

    gpu_procs
}

#[derive(Debug)]
struct GPUProcessInfo<'a> {
    device_path: &'a str,
    device: &'a gpu::Device,
    processes: Vec<&'a process::Process>,
}
