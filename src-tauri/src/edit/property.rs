use super::error::{EditError, EditResult};
use super::rsa_keys::read_key_file;
use super::types::LogSink;

const PADDING_BYTE: u8 = 0x20;

pub fn replace_tibia_rsa_key(log: &mut LogSink, tibia_binary: &mut Vec<u8>) -> EditResult<()> {
    log.info("Loading RSA keys...");
    let tibia_rsa = read_key_file("tibia_rsa.key")?;
    let otserv_rsa = read_key_file("otserv_rsa.key")?;

    log.info("Searching for Tibia RSA...");

    if tibia_binary
        .windows(tibia_rsa.len())
        .any(|w| w == tibia_rsa.as_slice())
    {
        log.info("Tibia RSA found!");
        replace_first_occurrence(tibia_binary, &tibia_rsa, &otserv_rsa);
        log.patch("Tibia RSA replaced with OTServ RSA!");
    } else if tibia_binary
        .windows(otserv_rsa.len())
        .any(|w| w == otserv_rsa.as_slice())
    {
        log.warn("OTServ RSA already patched!");
    } else {
        return Err(EditError::RsaNotFound);
    }
    Ok(())
}

fn replace_first_occurrence(haystack: &mut Vec<u8>, from: &[u8], to: &[u8]) {
    if let Some(pos) = haystack.windows(from.len()).position(|w| w == from) {
        haystack.splice(pos..pos + from.len(), to.iter().copied());
    }
}

pub fn set_property_by_name(
    log: &mut LogSink,
    tibia_binary: &mut Vec<u8>,
    property_name: &str,
    custom_value: &str,
) -> bool {
    let original_size = tibia_binary.len();
    let property_key = format!("{property_name}=");
    let Some(property_index) = find_subslice(tibia_binary, property_key.as_bytes()) else {
        log.warn(format!("{property_key} was not found!"));
        return false;
    };

    let start_value = property_index + property_key.len();
    let Some(newline_offset) = tibia_binary[start_value..].iter().position(|&b| b == b'\n') else {
        log.warn(format!("{property_key} value terminator not found!"));
        return false;
    };
    let end_value = start_value + newline_offset;
    let property_value = String::from_utf8_lossy(&tibia_binary[start_value..end_value]).to_string();

    if custom_value.len() > property_value.len() {
        log.error(format!(
            "Cannot replace {property_key} to '{custom_value}' because the new value must be smaller than '{property_value}' ({} chars).",
            property_value.len()
        ));
        return false;
    }

    log.info(format!("{property_key} found! {property_value}"));

    let mut padded = custom_value.as_bytes().to_vec();
    padded.extend(std::iter::repeat(PADDING_BYTE).take(property_value.len() - custom_value.len()));

    let remaining = tibia_binary[end_value..].to_vec();
    tibia_binary.truncate(start_value);
    tibia_binary.extend_from_slice(&padded);
    tibia_binary.extend_from_slice(&remaining);

    if tibia_binary.len() != original_size {
        log.error(format!(
            "Fatal error: The new modified client (size {}) has a different byte size from the original (size {original_size}).",
            tibia_binary.len()
        ));
        return false;
    }

    log.patch(format!("{property_key} replaced to {custom_value}!"));
    true
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
