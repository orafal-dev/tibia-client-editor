mod battleye;
mod client_check;
pub mod config;
mod error;
mod patterns;
mod pe;
mod property;
mod rsa_keys;
mod util;

pub use client_check::analyze_tibia_binary;
pub use error::{EditError, EditResult};
pub use types::{DiagnoseOutput, DiagnosisReport, EditOutput, LogSink};

mod types;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use client_check::{
    client_check_code_reference_count, client_check_indicator_count, client_check_indicator_keys,
    has_unsafe_client_check_remainder, is_partial_client_check_support,
    is_warning_client_check_support, log_battleye_signature_report,
    log_client_check_support_summary, patch_state_by_name, strong_unsupported_evidence_count,
    strong_unsupported_evidence_keys, suspicious_active_evidence_keys,
    suspicious_active_indicator_keys,
};
use config::{resolve_config_values, sync_config_ini};
use patterns::{battleye_patches, patchable_battleye_patch_count};
use property::{replace_tibia_rsa_key, set_property_by_name};
use util::{difference_strings, resolve_source_executable};

pub fn edit_client(
    tibia_exe: &Path,
    config_path: Option<&Path>,
    config_values: Option<std::collections::HashMap<String, String>>,
    source_tibia_exe: Option<&Path>,
    strict_client_check: bool,
    aggressive_client_check: bool,
) -> EditResult<EditOutput> {
    let mut log = LogSink::new();
    let config_values = resolve_config_values(config_path, config_values)?;

    let tibia_path = tibia_exe.to_path_buf();
    let source_path = resolve_source_executable(&tibia_path, source_tibia_exe);
    let source_binary = std::fs::read(&source_path).map_err(EditError::Io)?;
    let mut tibia_binary = source_binary.clone();
    let original_binary_size = source_binary.len();
    let original_tibia_binary = source_binary.clone();

    if source_path != tibia_path {
        log.info(format!(
            "Using source client executable for patch input: {}",
            source_path.file_name().unwrap().to_string_lossy()
        ));
        log.info(format!(
            "Writing patched client to target executable: {}",
            tibia_path.file_name().unwrap().to_string_lossy()
        ));
    }

    replace_tibia_rsa_key(&mut log, &mut tibia_binary)?;
    battleye::remove_battleye(
        &mut log,
        tibia_path.to_str().unwrap_or(""),
        &mut tibia_binary,
        aggressive_client_check,
    )?;

    let diagnosis = analyze_tibia_binary(tibia_path.to_str().unwrap_or(""), &tibia_binary);
    log_client_check_support_summary(&mut log, &diagnosis);
    enforce_edit_client_check_policy(&mut log, &diagnosis, strict_client_check)?;

    let mut properties_patched = Vec::new();
    let mut properties_failed = Vec::new();
    for (prop, value) in &config_values {
        if set_property_by_name(&mut log, &mut tibia_binary, prop, value) {
            properties_patched.push(prop.clone());
        } else {
            properties_failed.push(prop.clone());
            log.error(format!("Unable to replace {prop}"));
        }
    }

    let backup_binary = if source_path != tibia_path {
        std::fs::read(&tibia_path).unwrap_or(original_tibia_binary.clone())
    } else {
        original_tibia_binary.clone()
    };

    let backup_path = backup_executable(
        &mut log,
        &tibia_path,
        &backup_binary,
        aggressive_client_check,
    )?;
    export_modified_file(&mut log, &tibia_path, &tibia_binary, original_binary_size)?;
    sync_config_ini(
        &mut log,
        &tibia_path,
        &original_tibia_binary,
        &config_values,
    )?;

    log_edit_success(&mut log, &diagnosis, strict_client_check);

    Ok(EditOutput {
        logs: log.messages,
        diagnosis,
        success: true,
        output_path: tibia_path.display().to_string(),
        backup_path: Some(backup_path),
        properties_patched,
        properties_failed,
    })
}

pub fn diagnose_client(
    tibia_exe: &Path,
    compare_with: Option<&Path>,
    strict_client_check: bool,
) -> EditResult<DiagnoseOutput> {
    let mut log = LogSink::new();
    let target_data = std::fs::read(tibia_exe).map_err(EditError::Io)?;
    let target = analyze_tibia_binary(tibia_exe.to_str().unwrap_or(""), &target_data);
    print_diagnosis_report(&mut log, &target, "target");

    let mut baseline = None;
    let mut comparison_logs = None;

    if let Some(compare_path) = compare_with {
        let compare_data = std::fs::read(compare_path).map_err(EditError::Io)?;
        let compare_diagnosis =
            analyze_tibia_binary(compare_path.to_str().unwrap_or(""), &compare_data);
        print_diagnosis_report(&mut log, &compare_diagnosis, "baseline");
        let mut cmp_log = LogSink::new();
        print_diagnosis_comparison(&mut cmp_log, &compare_diagnosis, &target);
        comparison_logs = Some(cmp_log.messages);
        baseline = Some(compare_diagnosis);
    }

    if strict_client_check && has_unsafe_client_check_remainder(&target) {
        return Err(EditError::msg(format!(
            "Refusing to continue because strict client-check validation is enabled and the verdict is {}",
            target.client_check_verdict
        )));
    }

    Ok(DiagnoseOutput {
        logs: log.messages,
        target,
        baseline,
        comparison_logs,
    })
}

fn backup_executable(
    log: &mut LogSink,
    tibia_path: &Path,
    tibia_binary: &[u8],
    aggressive: bool,
) -> EditResult<String> {
    let file_name = tibia_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "client.exe".to_string());
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_path = tibia_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("BKP{timestamp}-{file_name}"));

    if aggressive {
        for line in [
            "============================================================",
            "AGGRESSIVE MODE IS ENABLED",
            "High-risk signatures are being rewritten automatically.",
            "This mode can break runtime behavior and can crash or fail to start some clients.",
            "Create/keep a known-good backup before using it.",
            "This may alter client behavior and should only be used with full manual validation.",
            "============================================================",
        ] {
            log.warn(line);
        }
    }

    log.info(format!(
        "Backing up {file_name} to {}",
        backup_path.file_name().unwrap().to_string_lossy()
    ));
    std::fs::write(&backup_path, tibia_binary).map_err(EditError::Io)?;
    Ok(backup_path.display().to_string())
}

fn export_modified_file(
    log: &mut LogSink,
    tibia_path: &Path,
    tibia_binary: &[u8],
    original_binary_size: usize,
) -> EditResult<()> {
    if tibia_binary.len() != original_binary_size {
        return Err(EditError::InvalidPatchSize(
            original_binary_size,
            tibia_binary.len(),
        ));
    }
    std::fs::write(tibia_path, tibia_binary).map_err(EditError::Io)?;
    log.info(format!(
        "Patched file exported to: {}",
        tibia_path.display()
    ));
    Ok(())
}

fn enforce_edit_client_check_policy(
    log: &mut LogSink,
    diagnosis: &DiagnosisReport,
    strict_client_check: bool,
) -> EditResult<()> {
    let verdict = &diagnosis.client_check_verdict;
    if diagnosis.strong_unsupported_evidence_count > 0 {
        log.error(format!(
            "UNSUPPORTED support - refusing export because strong client-check evidence remains ({} code reference(s))",
            diagnosis.strong_unsupported_evidence_count
        ));
        log.error(format!("Verdict: {verdict}"));
        log.error("Run diagnose and inspect the Strong unsupported evidence section before using this client");
        return Err(EditError::UnsupportedClientCheck(
            diagnosis.strong_unsupported_evidence_count,
        ));
    }

    if is_partial_client_check_support(diagnosis) {
        if strict_client_check {
            log.error("PARTIAL support - refusing export because strict mode is enabled");
            log.error(format!("Verdict: {verdict}"));
            log.error("Re-run without strict mode only if this partial support is acceptable for manual testing");
            return Err(EditError::PartialStrict);
        }
        log.warn("PARTIAL support - client may work but not fully verified");
        log.warn(format!("Verdict: {verdict}"));
        return Ok(());
    }

    if is_warning_client_check_support(diagnosis) {
        if strict_client_check {
            log.error("WARNING support - refusing export because strict mode is enabled");
            log.error(format!("Verdict: {verdict}"));
            log.error("Re-run diagnose and inspect Suspicious active client-check candidates before using this client");
            return Err(EditError::WarningStrict);
        }
        log.warn(
            "WARNING support - client-check branch/call candidates remain after the known patch",
        );
        log.warn("Client-check paths may still be active. Test recommended.");
        log.warn(format!("Verdict: {verdict}"));
        return Ok(());
    }

    log.info(format!("Client-check edit gate: {verdict}"));
    Ok(())
}

fn log_edit_success(log: &mut LogSink, diagnosis: &DiagnosisReport, strict_client_check: bool) {
    if is_partial_client_check_support(diagnosis) {
        log.warn(format!(
            "Edit completed with PARTIAL support - client may work but not fully verified (strict={strict_client_check})"
        ));
        return;
    }
    if is_warning_client_check_support(diagnosis) {
        log.warn(format!(
            "Edit completed with WARNING support - suspicious client-check branch/call candidates remain (strict={strict_client_check})"
        ));
        log.warn("Client-check paths may still be active. Test recommended.");
        return;
    }
    log.info(format!(
        "Edit completed with {}",
        diagnosis.client_check_verdict
    ));
}

fn print_diagnosis_report(log: &mut LogSink, diagnosis: &DiagnosisReport, label: &str) {
    log.info(format!("Diagnosing {label}: {}", diagnosis.path));
    log.info(format!("Size: {} bytes", diagnosis.size));
    log.info(format!("SHA256: {}", diagnosis.sha256));

    if !diagnosis.is_windows_exe {
        log.warn("This file is not a Windows PE executable; BattlEye byte patch signatures are informational only");
    }
    if diagnosis.is_windows_exe && !diagnosis.pe.valid {
        log.warn(format!(
            "PE section parsing failed; code-reference diagnostics are unavailable: {}",
            diagnosis.pe.error_text.as_deref().unwrap_or("unknown")
        ));
    }

    log_battleye_signature_report(log, diagnosis);
    log_client_check_support_summary(log, diagnosis);
}

fn print_diagnosis_comparison(
    log: &mut LogSink,
    baseline: &DiagnosisReport,
    target: &DiagnosisReport,
) {
    log.info(format!(
        "Comparative diagnosis: baseline={} target={}",
        baseline.path, target.path
    ));
    log.info(format!(
        "Size delta: {:+} bytes",
        target.size as i64 - baseline.size as i64
    ));
    if baseline.sha256 == target.sha256 {
        log.info("SHA256: identical");
    } else {
        log.info(format!(
            "SHA256: baseline={} target={}",
            baseline.sha256, target.sha256
        ));
    }

    log.info(format!(
        "Known patch coverage: baseline={}/{} target={}/{}",
        baseline.known_patch_coverage,
        patchable_battleye_patch_count(),
        target.known_patch_coverage,
        patchable_battleye_patch_count(),
    ));

    for patch in battleye_patches() {
        log.info(format!(
            "Patch {:?}: baseline={} target={}",
            patch.name,
            patch_state_by_name(baseline, patch.name),
            patch_state_by_name(target, patch.name),
        ));
    }

    log.info(format!(
        "Client-check indicators: baseline={} target={}",
        client_check_indicator_count(baseline),
        client_check_indicator_count(target),
    ));
    log.info(format!(
        "Client-check code refs: baseline={} target={}",
        client_check_code_reference_count(baseline),
        client_check_code_reference_count(target),
    ));
    log.info(format!(
        "Strong unsupported evidence: baseline={} target={}",
        strong_unsupported_evidence_count(baseline),
        strong_unsupported_evidence_count(target),
    ));
    log.info(format!(
        "Suspicious active candidates: baseline={} target={}",
        baseline.suspicious_active_evidence_count, target.suspicious_active_evidence_count,
    ));

    let new_indicators = difference_strings(
        &client_check_indicator_keys(target),
        &client_check_indicator_keys(baseline),
    );
    if !new_indicators.is_empty() {
        log.warn(format!(
            "New target-only client-check indicators: {}",
            new_indicators.join(", ")
        ));
    }

    let new_strong = difference_strings(
        &strong_unsupported_evidence_keys(target),
        &strong_unsupported_evidence_keys(baseline),
    );
    if !new_strong.is_empty() {
        log.error(format!(
            "Target-only strong unsupported evidence: {}",
            new_strong.join("; ")
        ));
    }

    let new_suspicious = difference_strings(
        &suspicious_active_evidence_keys(target),
        &suspicious_active_evidence_keys(baseline),
    );
    if !new_suspicious.is_empty() {
        log.warn(format!(
            "Target-only suspicious active candidates: {}",
            new_suspicious.join("; ")
        ));
    }

    let _ = suspicious_active_indicator_keys;
}
