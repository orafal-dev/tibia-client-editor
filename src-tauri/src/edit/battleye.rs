use super::client_check::{analyze_tibia_binary, has_client_check_string_indicators};
use super::error::{EditError, EditResult};
use super::patterns::{
    battleye_patches, new_byte_pattern, patchable_battleye_patch_count, BattleyePatch,
    PATCH_CONTEXT_RADIUS, WILDCARD_BYTE,
};
use super::types::LogSink;
use super::util::{bytes_around_range, format_bytes, format_offsets_limited, is_windows_executable};

pub fn remove_battleye(
    log: &mut LogSink,
    _tibia_path: &str,
    tibia_binary: &mut Vec<u8>,
    aggressive: bool,
) -> EditResult<()> {
    if !is_windows_executable(tibia_binary) {
        log.warn("Battleye patch skipped because the client is not a Windows executable");
        return Ok(());
    }

    log.info("Searching for BattlEye byte patch signatures...");
    if aggressive {
        log.warn("Aggressive mode enabled: high-risk signatures are eligible for patching.");
    }

    let active_patches: Vec<BattleyePatch> = battleye_patches()
        .into_iter()
        .map(|p| p.with_aggressive_mode(aggressive))
        .collect();

    let mut patches_applied = 0;
    let mut signatures_applied = 0;
    let mut already_applied = 0;
    let patchable_signatures = active_patches
        .iter()
        .filter(|p| !p.diagnostic_only || (aggressive && !p.aggressive_replacement.is_empty()))
        .count();

    for patch in active_patches {
        let original_offsets = patch.original.find_all(tibia_binary);
        let patched_offsets = patch.patched.find_all(tibia_binary);

        if patch.diagnostic_only {
            if !original_offsets.is_empty() && aggressive && !patch.aggressive_replacement.is_empty() {
                let mut aggressive_patch = patch.clone();
                aggressive_patch.diagnostic_only = false;
                aggressive_patch.replacement = patch.aggressive_replacement.clone();
                aggressive_patch.patched = new_byte_pattern(
                    &format!("{} [aggressive]", patch.name),
                    &patch.aggressive_replacement,
                );
                let count = original_offsets.len();
                apply_battleye_patch(log, tibia_binary, &aggressive_patch, &original_offsets)?;
                patches_applied += count;
                signatures_applied += 1;
                log.patch(format!(
                    "BattlEye high-risk signature {:?} patched aggressively ({count} occurrence(s))",
                    patch.name
                ));
                continue;
            }
            if !original_offsets.is_empty() || !patched_offsets.is_empty() {
                log.info(format!(
                    "BattlEye diagnostic signature {:?} found original={} patched={}; not applied automatically",
                    patch.name,
                    format_offsets_limited(&original_offsets, 6),
                    format_offsets_limited(&patched_offsets, 6),
                ));
            }
            continue;
        }

        if !original_offsets.is_empty() {
            let count = original_offsets.len();
            apply_battleye_patch(log, tibia_binary, &patch, &original_offsets)?;
            patches_applied += count;
            signatures_applied += 1;
            log.patch(format!(
                "BattlEye signature {:?} patched ({count} occurrence(s))",
                patch.name
            ));
            continue;
        }

        let patched_count = patched_offsets.len();
        if patched_count > 0 {
            already_applied += patched_count;
            log.info(format!(
                "BattlEye signature {:?} already patched ({patched_count} occurrence(s))",
                patch.name
            ));
            continue;
        }

        log.info(format!("BattlEye signature {:?} not found", patch.name));
    }

    if patches_applied > 0 {
        log.patch(format!(
            "BattlEye byte patch summary: applied {patches_applied} occurrence(s) across {signatures_applied}/{patchable_signatures} patchable signature(s)"
        ));
        if signatures_applied < patchable_signatures {
            log.warn("BattlEye byte patch is partial for this binary; missing signatures can mean this client version uses different code paths");
        }
        if has_client_check_string_indicators(tibia_binary) {
            log.warn("Client-check strings remain after BattlEye patching; this edit should be treated as PARTIAL unless code-reference diagnostics prove the paths inactive");
        }
        return Ok(());
    }

    if already_applied > 0 {
        log.warn(format!(
            "BattlEye byte patches were already present ({already_applied} occurrence(s)); no new byte patch was applied"
        ));
        if has_client_check_string_indicators(tibia_binary) {
            log.warn("Client-check strings remain in an already patched binary; this should be treated as PARTIAL unless code-reference diagnostics prove the paths inactive");
        }
        return Ok(());
    }

    log.warn("BattlEye byte patch signatures not found");
    if has_client_check_string_indicators(tibia_binary) {
        log.warn("Client-check strings remain and no patchable BattlEye signature matched; this binary is likely unsupported by the current patch set");
    }
    Ok(())
}

fn apply_battleye_patch(
    log: &mut LogSink,
    tibia_binary: &mut [u8],
    patch: &BattleyePatch,
    offsets: &[usize],
) -> EditResult<()> {
    if patch.diagnostic_only {
        return Ok(());
    }
    if patch.replacement.len() != patch.original.data.len() {
        return Err(EditError::InvalidPatch(patch.name.to_string()));
    }

    for &offset in offsets {
        let (context_start, before_bytes) =
            bytes_around_range(tibia_binary, offset, patch.replacement.len(), PATCH_CONTEXT_RADIUS);
        for (index, &value) in patch.replacement.iter().enumerate() {
            if value == WILDCARD_BYTE {
                continue;
            }
            tibia_binary[offset + index] = value as u8;
        }
        let (_, after_bytes) =
            bytes_around_range(tibia_binary, offset, patch.replacement.len(), PATCH_CONTEXT_RADIUS);
        let context_end = context_start + before_bytes.len();
        log.info(format!(
            "  bytes before @0x{context_start:X}..0x{context_end:X}: {}",
            format_bytes(&before_bytes)
        ));
        log.info(format!(
            "  bytes after  @0x{context_start:X}..0x{context_end:X}: {}",
            format_bytes(&after_bytes)
        ));
    }
    Ok(())
}
