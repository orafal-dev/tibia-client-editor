use goblin::pe::PE;

use super::types::{PeInfo, PeSectionInfo};

pub fn inspect_pe(data: &[u8]) -> PeInfo {
    if !super::util::is_windows_executable(data) {
        return PeInfo {
            valid: false,
            error_text: Some("not a PE executable".to_string()),
            image_base: 0,
            sections: Vec::new(),
            imports: Vec::new(),
        };
    }

    match PE::parse(data) {
        Ok(pe) => {
            let image_base = pe.image_base as u64;
            let mut sections = Vec::new();
            for section in &pe.sections {
                let raw_start = section.pointer_to_raw_data as usize;
                let raw_size = section.size_of_raw_data as usize;
                let raw_end = raw_start.saturating_add(raw_size).min(data.len());
                if raw_start >= data.len() || raw_end <= raw_start {
                    continue;
                }
                let virtual_size = section.virtual_size as usize;
                let size = section.size_of_raw_data as usize;
                let virtual_size = virtual_size.max(size);
                let name = section.name().unwrap_or("").trim_end_matches('\0').to_string();
                let characteristics = section.characteristics;
                let is_code = characteristics & 0x00000020 != 0 || characteristics & 0x20000000 != 0;
                sections.push(PeSectionInfo {
                    name,
                    raw_start,
                    raw_end,
                    rva_start: section.virtual_address as usize,
                    rva_end: section.virtual_address as usize + virtual_size,
                    is_code,
                });
            }

            let mut imports = Vec::new();
            for entry in &pe.imports {
                imports.push(entry.dll.to_string());
                imports.push(entry.name.to_string());
            }
            imports.sort();
            imports.dedup();

            PeInfo {
                valid: true,
                error_text: None,
                image_base,
                sections,
                imports,
            }
        }
        Err(err) => PeInfo {
            valid: false,
            error_text: Some(err.to_string()),
            image_base: 0,
            sections: Vec::new(),
            imports: Vec::new(),
        },
    }
}

impl PeInfo {
    pub fn rva_for_offset(&self, offset: usize) -> Option<usize> {
        for section in &self.sections {
            if offset >= section.raw_start && offset < section.raw_end {
                return Some(section.rva_start + offset - section.raw_start);
            }
        }
        None
    }
}
