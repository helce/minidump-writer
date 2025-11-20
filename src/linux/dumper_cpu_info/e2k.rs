use crate::{errors::CpuInfoError, minidump_format::*};
use scroll::Pwrite;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path,
};

type Result<T> = std::result::Result<T, CpuInfoError>;

struct CpuInfoEntry {
    field: &'static str,
    value: i32,
}

impl CpuInfoEntry {
    fn new(field: &'static str, value: i32) -> Self {
        CpuInfoEntry { field, value }
    }
}

pub fn write_cpu_information(sys_info: &mut MDRawSystemInfo) -> Result<()> {
    let vendor_id_name = "vendor_id";
    let mut cpu_info_table = [
        CpuInfoEntry::new("processor", 0),
        CpuInfoEntry::new("model", 0),
        CpuInfoEntry::new("cpu family", 0),
        CpuInfoEntry::new("revision", 0),
    ];

    // processor_architecture should always be set, do this first
    sys_info.processor_architecture = MDCPUArchitecture::PROCESSOR_ARCHITECTURE_E2K as u16;
    let cpuinfo_file = File::open(path::PathBuf::from("/proc/cpuinfo"))?;
    let mut vendor_id = String::new();

    for line in BufReader::new(cpuinfo_file).lines() {
        let line = line?;
        // Expected format: <field-name> <space>+ ':' <space> <value>
        // Note that:
        //   - empty lines happen.
        //   - <field-name> can contain spaces.
        //   - some fields have an empty <value>
        if line.trim().is_empty() {
            continue;
        }

        let mut liter = line.split(':').map(|x| x.trim());
        let field = liter.next().unwrap(); // guaranteed to have at least one item

        if let Some(val) = liter.next() {
            for entry in cpu_info_table.iter_mut() {
                if field == entry.field {
                    if let Ok(v) = val.parse() {
                        entry.value = v;
                    } else {
                        continue;
                    }
                }
                // special case for vendor_id
                if field == vendor_id_name && !val.is_empty() {
                    vendor_id = val.into();
                }
            }
        }
    }

    // cpu_info_table[0] holds the last cpu id listed in /proc/cpuinfo,
    // assuming this is the highest id, change it to the number of CPUs
    // by adding one.
    cpu_info_table[0].value += 1;
    sys_info.number_of_processors = cpu_info_table[0].value as u8;
    sys_info.processor_level = (cpu_info_table[2].value << 16 | cpu_info_table[1].value) as u16;
    sys_info.processor_revision = cpu_info_table[3].value as u16;
    // The sys_info.cpu field is just a byte array, but in e2k's case it is
    // actually
    // minidump_common::format::E2KCpuInfo {
    //  pub vendor_id: [u32; 3],
    //  pub iset_id: u32,
    //  pub model_id: u32,
    //  pub revision_id: u32,
    // }
    let vendor_id = vendor_id.as_bytes();
    // The vendor_id is the first 12 (3 * size_of::<u32>()) bytes
    let vendor_len = std::cmp::min(3 * std::mem::size_of::<u32>(), vendor_id.len());
    sys_info.cpu.data[..vendor_len].copy_from_slice(&vendor_id[..vendor_len]);
    sys_info
        .cpu
        .data
        .pwrite_with(
            cpu_info_table[2].value as u32,
            3 * std::mem::size_of::<u32>(),
            scroll::Endian::Little,
        )
        .expect("impossible");
    sys_info
        .cpu
        .data
        .pwrite_with(
            cpu_info_table[1].value as u32,
            4 * std::mem::size_of::<u32>(),
            scroll::Endian::Little,
        )
        .expect("impossible");
    sys_info
        .cpu
        .data
        .pwrite_with(
            cpu_info_table[3].value as u32,
            5 * std::mem::size_of::<u32>(),
            scroll::Endian::Little,
        )
        .expect("impossible");

    Ok(())
}
