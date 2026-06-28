use std::collections::HashSet;

use super::patterns::{
    client_check_code_patterns, client_check_indicators, battleye_patches,
    CODE_CONTEXT_RADIUS, CONTEXT_BYTES_RADIUS, KNOWN_PATCH_CONTEXT_RADIUS,
    QT_CONTEXT_INDICATORS, patchable_battleye_patch_count,
};
use super::types::{
    BattleyePatchStatus, ClientCheckFinding, ClientCheckReference, DiagnosisReport,
    KnownPatchOffset, LogSink, PatternMatch, PeInfo, PeSectionInfo,
};
use super::util::{
    abs_distance, bytes_around, find_all_offsets, format_bytes, format_nearest_offsets,
    format_offsets_limited, is_windows_executable, utf16_le_bytes,
};

pub fn analyze_tibia_binary(path: &str, data: &[u8]) -> DiagnosisReport {
    use sha2::{Digest, Sha256};
    let sha256_text = hex::encode(Sha256::digest(data));
    let is_windows_exe = is_windows_executable(data);
    let patch_statuses = scan_battleye_patch_status(data, &sha256_text);
    let pe = if is_windows_exe {
        super::pe::inspect_pe(data)
    } else {
        super::types::PeInfo {
            valid: false,
            error_text: None,
            image_base: 0,
            sections: Vec::new(),
            imports: Vec::new(),
        }
    };
    let client_check_findings = scan_client_check_findings(data, &pe, &patch_statuses);
    let qt_indicators = scan_qt_context_indicators(data, &pe);

    let mut report = DiagnosisReport {
        path: path.to_string(),
        size: data.len(),
        sha256: sha256_text,
        is_windows_exe,
        pe,
        patch_statuses,
        client_check_findings,
        qt_indicators,
        client_check_verdict: String::new(),
        known_patch_coverage: 0,
        patchable_count: patchable_battleye_patch_count(),
        original_patch_signature_count: 0,
        patched_patch_signature_count: 0,
        strong_unsupported_evidence_count: 0,
        suspicious_active_evidence_count: 0,
        high_risk_diagnostic_count: 0,
    };
    fill_summary_fields(&mut report);
    report
}

fn fill_summary_fields(report: &mut DiagnosisReport) {
    report.client_check_verdict = client_check_verdict(report);
    report.known_patch_coverage = known_patch_coverage(report);
    report.original_patch_signature_count = original_patch_signature_count(report);
    report.patched_patch_signature_count = patched_patch_signature_count(report);
    report.strong_unsupported_evidence_count = strong_unsupported_evidence_count(report);
    report.suspicious_active_evidence_count = suspicious_active_evidence_count(report);
    report.high_risk_diagnostic_count = high_risk_client_check_diagnostic_count(report);
}

fn scan_battleye_patch_status(data: &[u8], sha256_text: &str) -> Vec<BattleyePatchStatus> {
    battleye_patches()
        .into_iter()
        .map(|patch| {
            let mut patched_offsets = patch.patched.find_all(data);
            if patch.diagnostic_only && !patch.aggressive_replacement.is_empty() {
                let aggressive = super::patterns::new_byte_pattern(
                    &format!("{} [aggressive]", patch.name),
                    &patch.aggressive_replacement,
                );
                patched_offsets.extend(aggressive.find_all(data));
            }
            BattleyePatchStatus {
                name: patch.name.to_string(),
                diagnostic_only: patch.diagnostic_only,
                high_risk_client_check: patch.high_risk_client_check,
                false_positive_check: patch.false_positive_check.to_string(),
                original_offset: patch.original.find_all(data),
                patched_offset: patched_offsets,
                expected_offset_hits: patch
                    .expected_offset_hits(data, sha256_text)
                    .into_iter()
                    .map(|e| KnownPatchOffset {
                        sha256: e.sha256.to_string(),
                        offset: e.offset,
                        note: e.note.to_string(),
                    })
                    .collect(),
                expected_offset_misses: patch
                    .expected_offset_misses(data, sha256_text)
                    .into_iter()
                    .map(|e| KnownPatchOffset {
                        sha256: e.sha256.to_string(),
                        offset: e.offset,
                        note: e.note.to_string(),
                    })
                    .collect(),
                aob_mask: patch.original.format_aob(),
            }
        })
        .collect()
}

fn scan_client_check_findings(
    data: &[u8],
    pe_data: &PeInfo,
    patch_statuses: &[BattleyePatchStatus],
) -> Vec<ClientCheckFinding> {
    let mut findings = Vec::new();
    for indicator in client_check_indicators() {
        findings = append_client_check_finding(
            findings,
            data,
            pe_data,
            patch_statuses,
            indicator.name,
            "ascii",
            indicator.value,
        );
        let utf16 = utf16_le_bytes(std::str::from_utf8(indicator.value).unwrap_or(""));
        if !utf16.is_empty() {
            findings = append_client_check_finding(
                findings,
                data,
                pe_data,
                patch_statuses,
                indicator.name,
                "utf16-le",
                &utf16,
            );
        }
    }
    findings
}

fn append_client_check_finding(
    mut findings: Vec<ClientCheckFinding>,
    data: &[u8],
    pe_data: &PeInfo,
    patch_statuses: &[BattleyePatchStatus],
    name: &str,
    encoding: &str,
    needle: &[u8],
) -> Vec<ClientCheckFinding> {
    let offsets = find_all_offsets(data, needle);
    if offsets.is_empty() {
        return findings;
    }
    let mut references = Vec::new();
    if pe_data.valid {
        for offset in &offsets {
            references.extend(find_string_code_references(
                data,
                pe_data,
                patch_statuses,
                name,
                *offset,
            ));
        }
    }
    findings.push(ClientCheckFinding {
        name: name.to_string(),
        encoding: encoding.to_string(),
        offsets,
        references,
    });
    findings
}

fn find_string_code_references(
    data: &[u8],
    pe_data: &PeInfo,
    patch_statuses: &[BattleyePatchStatus],
    indicator_name: &str,
    string_offset: usize,
) -> Vec<ClientCheckReference> {
    let Some(string_rva) = pe_data.rva_for_offset(string_offset) else {
        return Vec::new();
    };
    let mut references = Vec::new();
    for section in &pe_data.sections {
        if !section.is_code {
            continue;
        }
        let mut offset = section.raw_start;
        while offset < section.raw_end {
            if let Some((instruction_length, instruction_name, displacement_offset)) =
                rip_relative_instruction_at(data, section.raw_end, offset)
            {
                let Some(instruction_rva) = pe_data.rva_for_offset(offset) else {
                    offset += 1;
                    continue;
                };
                let displacement = i32::from_le_bytes(
                    data[displacement_offset..displacement_offset + 4]
                        .try_into()
                        .unwrap(),
                ) as i64;
                let target_rva = instruction_rva as i64 + instruction_length as i64 + displacement;
                if target_rva as usize == string_rva {
                    let reference = enrich_code_reference_context(
                        data,
                        section,
                        ClientCheckReference {
                            offset,
                            section: section.name.clone(),
                            instruction: instruction_name,
                            branch_offsets: Vec::new(),
                            call_offsets: Vec::new(),
                            pattern_matches: Vec::new(),
                            context_start: 0,
                            context_bytes: Vec::new(),
                            known_patch_nearby: false,
                            strong_unsupported: false,
                            suspicious_active: false,
                            possible_instructions: String::new(),
                        },
                        patch_statuses,
                        indicator_name,
                    );
                    references.push(reference);
                }
            }
            offset += 1;
        }
    }
    dedupe_adjacent_rex_references(references)
}

fn dedupe_adjacent_rex_references(references: Vec<ClientCheckReference>) -> Vec<ClientCheckReference> {
    let mut deduped: Vec<ClientCheckReference> = Vec::new();
    for reference in references {
        if let Some(previous) = deduped.last() {
            if reference.offset == previous.offset + 1
                && previous.instruction.contains("REX:")
                && (reference.instruction == "LEA" || reference.instruction == "MOV")
            {
                continue;
            }
        }
        deduped.push(reference);
    }
    deduped
}

fn rip_relative_instruction_at(
    data: &[u8],
    section_end: usize,
    offset: usize,
) -> Option<(usize, String, usize)> {
    if offset >= section_end {
        return None;
    }
    let mut opcode_offset = offset;
    let mut rex_prefix = 0u8;
    if data[opcode_offset] & 0xf0 == 0x40 {
        rex_prefix = data[opcode_offset];
        opcode_offset += 1;
    }
    if opcode_offset + 6 > section_end || opcode_offset + 6 > data.len() {
        return None;
    }
    let opcode = data[opcode_offset];
    if opcode != 0x8d && opcode != 0x8b {
        return None;
    }
    let mod_rm = data[opcode_offset + 1];
    if mod_rm & 0xc7 != 0x05 {
        return None;
    }
    let instruction_length = opcode_offset - offset + 6;
    let mut instruction_name = if opcode == 0x8d {
        "LEA".to_string()
    } else {
        "MOV".to_string()
    };
    if rex_prefix != 0 {
        instruction_name = format!("{instruction_name} REX:{rex_prefix:02X}");
    }
    Some((instruction_length, instruction_name, opcode_offset + 2))
}

fn enrich_code_reference_context(
    data: &[u8],
    section: &PeSectionInfo,
    mut reference: ClientCheckReference,
    patch_statuses: &[BattleyePatchStatus],
    indicator_name: &str,
) -> ClientCheckReference {
    let window_start = reference.offset.saturating_sub(CODE_CONTEXT_RADIUS).max(section.raw_start);
    let window_end = (reference.offset + CODE_CONTEXT_RADIUS).min(section.raw_end);
    reference.branch_offsets = find_conditional_branches(data, window_start, window_end);
    reference.call_offsets = find_calls(data, window_start, window_end);
    reference.pattern_matches = find_code_pattern_matches(data, window_start, window_end);
    reference.known_patch_nearby =
        has_known_patch_nearby(patch_statuses, &reference.context_offsets(), KNOWN_PATCH_CONTEXT_RADIUS);
    let (context_start, context_bytes) = bytes_around(data, reference.offset, CONTEXT_BYTES_RADIUS);
    reference.context_start = context_start;
    reference.context_bytes = context_bytes;
    reference.strong_unsupported = is_strong_client_check_evidence(indicator_name, &reference);
    reference.suspicious_active = is_suspicious_active_client_check_evidence(indicator_name, &reference);
    reference.possible_instructions = format_possible_instructions(&reference);
    reference
}

impl ClientCheckReference {
    fn context_offsets(&self) -> Vec<usize> {
        let mut offsets = vec![self.offset];
        offsets.extend(&self.branch_offsets);
        offsets.extend(&self.call_offsets);
        offsets.extend(self.pattern_matches.iter().map(|m| m.offset));
        offsets
    }
}

fn find_conditional_branches(data: &[u8], start: usize, end: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    let end = end.min(data.len());
    for offset in start..end {
        if is_conditional_jump(data, offset, end) {
            offsets.push(offset);
        }
    }
    offsets
}

fn is_conditional_jump(data: &[u8], offset: usize, end: usize) -> bool {
    if offset >= end || offset >= data.len() {
        return false;
    }
    let opcode = data[offset];
    if (0x70..=0x7f).contains(&opcode) {
        return true;
    }
    offset + 1 < end
        && offset + 1 < data.len()
        && opcode == 0x0f
        && (0x80..=0x8f).contains(&data[offset + 1])
}

fn find_calls(data: &[u8], start: usize, end: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    let end = end.min(data.len());
    let mut offset = start;
    while offset < end {
        if offset + 5 <= end && data[offset] == 0xe8 {
            offsets.push(offset);
        } else if offset + 2 <= end && data[offset] == 0xff && data[offset + 1] & 0x38 == 0x10 {
            offsets.push(offset);
        }
        offset += 1;
    }
    offsets
}

fn find_code_pattern_matches(data: &[u8], start: usize, end: usize) -> Vec<PatternMatch> {
    let start = start.min(data.len());
    let end = end.min(data.len());
    if end <= start {
        return Vec::new();
    }
    let window = &data[start..end];
    let mut matches = Vec::new();
    for pattern in client_check_code_patterns() {
        for offset in pattern.find_all(window) {
            matches.push(PatternMatch {
                name: pattern.name.clone(),
                offset: start + offset,
            });
        }
    }
    matches
}

fn has_known_patch_nearby(
    patch_statuses: &[BattleyePatchStatus],
    reference_offsets: &[usize],
    radius: usize,
) -> bool {
    for status in patch_statuses {
        let mut known: Vec<usize> = status.original_offset.clone();
        known.extend(&status.patched_offset);
        for known_offset in known {
            for reference_offset in reference_offsets {
                if abs_distance(*reference_offset, known_offset) <= radius {
                    return true;
                }
            }
        }
    }
    false
}

fn is_strong_client_check_evidence(indicator_name: &str, reference: &ClientCheckReference) -> bool {
    if !is_critical_client_check_indicator(indicator_name) {
        return false;
    }
    if reference.known_patch_nearby {
        return false;
    }
    if reference.branch_offsets.is_empty() {
        return false;
    }
    !reference.pattern_matches.is_empty()
}

fn is_suspicious_active_client_check_evidence(
    indicator_name: &str,
    reference: &ClientCheckReference,
) -> bool {
    if reference.strong_unsupported {
        return false;
    }
    if !is_critical_client_check_indicator(indicator_name) {
        return false;
    }
    !reference.branch_offsets.is_empty() && !reference.call_offsets.is_empty()
}

fn is_critical_client_check_indicator(indicator_name: &str) -> bool {
    matches!(
        indicator_name,
        "clientcheck_disconnected"
            | "onCloseDueToClientCheckRequested"
            | "onClientCheckDialogButtonClicked"
            | "enableClientCheck"
            | "requestCloseDueToClientCheck"
    )
}

fn scan_qt_context_indicators(data: &[u8], pe_data: &PeInfo) -> Vec<String> {
    let mut seen = HashSet::new();
    for indicator in QT_CONTEXT_INDICATORS {
        if data.windows(indicator.len()).any(|w| w == indicator.as_bytes()) {
            seen.insert(format!("{indicator} string"));
        }
        let lower = indicator.to_lowercase();
        for imported in &pe_data.imports {
            if imported.to_lowercase().contains(&lower) {
                seen.insert(format!("{indicator} import"));
            }
        }
    }
    let mut indicators: Vec<String> = seen.into_iter().collect();
    indicators.sort();
    indicators
}

pub fn client_check_verdict(report: &DiagnosisReport) -> String {
    if strong_unsupported_evidence_count(report) > 0 {
        return "UNSUPPORTED: client-check code evidence remains".to_string();
    }
    if has_patched_client_check_signature(report) && high_risk_client_check_diagnostic_count(report) > 0 {
        return "WARNING: high risk of client-check remaining after known patch".to_string();
    }
    if has_patched_client_check_signature(report) && suspicious_active_evidence_count(report) > 0 {
        return "WARNING: known client-check patch applied but suspicious branch/call evidence remains"
            .to_string();
    }
    let coverage = known_patch_coverage(report);
    let patchable = patchable_battleye_patch_count();
    if coverage < patchable {
        return "PARTIAL: only some known patchable signatures are covered".to_string();
    }
    "SUPPORTED: all known patchable signatures are covered and no strong client-check evidence remains"
        .to_string()
}

pub fn known_patch_coverage(report: &DiagnosisReport) -> usize {
    report
        .patch_statuses
        .iter()
        .filter(|s| !s.diagnostic_only)
        .filter(|s| !s.original_offset.is_empty() || !s.patched_offset.is_empty())
        .count()
}

pub fn original_patch_signature_count(report: &DiagnosisReport) -> usize {
    report
        .patch_statuses
        .iter()
        .filter(|s| !s.diagnostic_only && !s.original_offset.is_empty())
        .count()
}

pub fn patched_patch_signature_count(report: &DiagnosisReport) -> usize {
    report
        .patch_statuses
        .iter()
        .filter(|s| !s.diagnostic_only && !s.patched_offset.is_empty())
        .count()
}

pub fn has_patched_client_check_signature(report: &DiagnosisReport) -> bool {
    patched_patch_signature_count(report) > 0
}

pub fn high_risk_client_check_diagnostic_count(report: &DiagnosisReport) -> usize {
    report
        .patch_statuses
        .iter()
        .filter(|s| s.diagnostic_only && s.high_risk_client_check)
        .filter(|s| !s.original_offset.is_empty() || !s.patched_offset.is_empty())
        .count()
}

pub fn strong_unsupported_evidence_count(report: &DiagnosisReport) -> usize {
    report
        .client_check_findings
        .iter()
        .flat_map(|f| &f.references)
        .filter(|r| r.strong_unsupported)
        .count()
}

pub fn suspicious_active_evidence_count(report: &DiagnosisReport) -> usize {
    report
        .client_check_findings
        .iter()
        .flat_map(|f| &f.references)
        .filter(|r| r.suspicious_active)
        .count()
}

pub fn is_partial_client_check_support(report: &DiagnosisReport) -> bool {
    report.client_check_verdict.starts_with("PARTIAL:")
}

pub fn is_warning_client_check_support(report: &DiagnosisReport) -> bool {
    report.client_check_verdict.starts_with("WARNING:")
}

pub fn has_unsafe_client_check_remainder(report: &DiagnosisReport) -> bool {
    is_partial_client_check_support(report)
        || is_warning_client_check_support(report)
        || strong_unsupported_evidence_count(report) > 0
}

pub fn has_client_check_string_indicators(data: &[u8]) -> bool {
    for indicator in client_check_indicators() {
        if data.windows(indicator.value.len()).any(|w| w == indicator.value) {
            return true;
        }
        let utf16 = utf16_le_bytes(std::str::from_utf8(indicator.value).unwrap_or(""));
        if !utf16.is_empty() && data.windows(utf16.len()).any(|w| w == utf16.as_slice()) {
            return true;
        }
    }
    false
}

pub fn log_client_check_support_summary(log: &mut LogSink, report: &DiagnosisReport) {
    log.info(format!("Client-check support verdict: {}", report.client_check_verdict));
    log.info(format!(
        "Known byte-patch coverage: {}/{} signature(s), original={}, patched={}",
        report.known_patch_coverage,
        report.patchable_count,
        report.original_patch_signature_count,
        report.patched_patch_signature_count,
    ));
    if report.client_check_findings.is_empty() {
        log.info("No known client-check string indicators remain");
        return;
    }
    log_strong_unsupported_evidence(log, report);
    log_suspicious_active_evidence(log, report);
    log_weak_indicators(log, report);
    if !report.qt_indicators.is_empty() {
        log.info(format!("Qt context indicators: {}", report.qt_indicators.join(", ")));
    }
    if report.strong_unsupported_evidence_count > 0 {
        log.error(format!(
            "Strong unsupported client-check evidence remains: {} code reference(s) combine a critical client-check string, nearby conditional branch, recognized branch/call pattern, and no known patch signature nearby",
            report.strong_unsupported_evidence_count
        ));
    }
    if has_patched_client_check_signature(report) && report.high_risk_diagnostic_count > 0 {
        log.warn(format!(
            "High-risk diagnostic-only client-check paths remain after a known patch was applied: {} signature(s)",
            report.high_risk_diagnostic_count
        ));
    }
    if has_patched_client_check_signature(report) && report.suspicious_active_evidence_count > 0 {
        log.warn(format!(
            "Suspicious active client-check branch/call evidence remains after a known patch was applied: {} code reference(s)",
            report.suspicious_active_evidence_count
        ));
    }
}

fn log_strong_unsupported_evidence(log: &mut LogSink, report: &DiagnosisReport) {
    if report.strong_unsupported_evidence_count == 0 {
        log.info("Strong unsupported evidence: none");
        return;
    }
    log.error("Strong unsupported evidence:");
    for finding in &report.client_check_findings {
        for reference in &finding.references {
            if !reference.strong_unsupported {
                continue;
            }
            log.error(format!(
                "  {:?} ({}) string={} ref={} at 0x{:X} in {} branches={} calls={} patterns={} knownPatchNearby={} context48={} possibleInstructions={}",
                finding.name,
                finding.encoding,
                format_offsets_limited(&finding.offsets, 4),
                reference.instruction,
                reference.offset,
                reference.section,
                format_nearest_offsets(reference.offset, &reference.branch_offsets, 6),
                format_nearest_offsets(reference.offset, &reference.call_offsets, 6),
                format_pattern_matches(&reference.pattern_matches, 4),
                reference.known_patch_nearby,
                format_bytes(&reference.context_bytes),
                reference.possible_instructions,
            ));
        }
    }
}

fn log_suspicious_active_evidence(log: &mut LogSink, report: &DiagnosisReport) {
    if report.suspicious_active_evidence_count == 0 {
        log.info("Suspicious active client-check candidates: none");
        return;
    }
    log.warn("Suspicious active client-check candidates:");
    for finding in &report.client_check_findings {
        for reference in &finding.references {
            if !reference.suspicious_active {
                continue;
            }
            log.warn(format!(
                "  {:?} ({}) string={} ref={} at 0x{:X} in {} nearestBranches={} nearestCalls={} patterns={} knownPatchNearby={} reason={} context48={} possibleInstructions={}",
                finding.name,
                finding.encoding,
                format_offsets_limited(&finding.offsets, 4),
                reference.instruction,
                reference.offset,
                reference.section,
                format_nearest_offsets(reference.offset, &reference.branch_offsets, 8),
                format_nearest_offsets(reference.offset, &reference.call_offsets, 8),
                format_pattern_matches(&reference.pattern_matches, 4),
                reference.known_patch_nearby,
                suspicious_evidence_reason(&finding.name, reference),
                format_bytes(&reference.context_bytes),
                reference.possible_instructions,
            ));
        }
    }
}

fn log_weak_indicators(log: &mut LogSink, report: &DiagnosisReport) {
    log.warn("Weak indicators:");
    for finding in &report.client_check_findings {
        if finding.references.is_empty() {
            log.warn(format!(
                "  {:?} ({}) string={} refs=none reason=no code xref found",
                finding.name,
                finding.encoding,
                format_offsets_limited(&finding.offsets, 8),
            ));
            continue;
        }
        for reference in &finding.references {
            if reference.strong_unsupported || reference.suspicious_active {
                continue;
            }
            log.warn(format!(
                "  {:?} ({}) string={} ref={} at 0x{:X} in {} nearestBranches={} nearestCalls={} patterns={} knownPatchNearby={} reason={} context48={} possibleInstructions={}",
                finding.name,
                finding.encoding,
                format_offsets_limited(&finding.offsets, 4),
                reference.instruction,
                reference.offset,
                reference.section,
                format_nearest_offsets(reference.offset, &reference.branch_offsets, 8),
                format_nearest_offsets(reference.offset, &reference.call_offsets, 8),
                format_pattern_matches(&reference.pattern_matches, 4),
                reference.known_patch_nearby,
                weak_evidence_reason(&finding.name, reference),
                format_bytes(&reference.context_bytes),
                reference.possible_instructions,
            ));
        }
    }
}

fn format_pattern_matches(matches: &[PatternMatch], limit: usize) -> String {
    if matches.is_empty() {
        return "none".to_string();
    }
    let (display, truncated) = if limit > 0 && matches.len() > limit {
        (&matches[..limit], matches.len() - limit)
    } else {
        (matches, 0)
    };
    let formatted: Vec<String> = display
        .iter()
        .map(|m| format!("{}@0x{:X}", m.name, m.offset))
        .collect();
    let mut result = formatted.join(", ");
    if truncated > 0 {
        result.push_str(&format!(", ... +{truncated} more"));
    }
    result
}

fn suspicious_evidence_reason(indicator_name: &str, reference: &ClientCheckReference) -> String {
    if !is_critical_client_check_indicator(indicator_name) {
        return "non-critical indicator".to_string();
    }
    if reference.branch_offsets.is_empty() {
        return "no conditional branch in context".to_string();
    }
    if reference.call_offsets.is_empty() {
        return "no call in context".to_string();
    }
    if reference.known_patch_nearby {
        return "critical indicator has nearby branch and call; known nearby signature lowers this from strong to warning".to_string();
    }
    if reference.pattern_matches.is_empty() {
        return "critical indicator has nearby branch and call but no recognized branch/call signature".to_string();
    }
    "candidate remains below strong-evidence threshold".to_string()
}

fn weak_evidence_reason(indicator_name: &str, reference: &ClientCheckReference) -> String {
    if !is_critical_client_check_indicator(indicator_name) {
        return "non-critical indicator".to_string();
    }
    if reference.known_patch_nearby {
        return "known patch signature nearby".to_string();
    }
    if reference.branch_offsets.is_empty() {
        return "no conditional branch in context".to_string();
    }
    if reference.call_offsets.is_empty() {
        return "no call in context".to_string();
    }
    if reference.pattern_matches.is_empty() {
        return "no recognized branch/call pattern in context".to_string();
    }
    "not escalated".to_string()
}

fn format_possible_instructions(reference: &ClientCheckReference) -> String {
    if reference.context_bytes.is_empty() {
        return "none".to_string();
    }
    let mut instructions = Vec::new();
    let mut index = 0;
    while index < reference.context_bytes.len() {
        let offset = reference.context_start + index;
        let data = &reference.context_bytes[index..];
        if data.len() >= 7 && data[0] == 0x48 && data[1] == 0x8d && (data[2] == 0x15 || data[2] == 0x0d) {
            let register = if data[2] == 0x15 { "rdx" } else { "rcx" };
            let displacement = i32::from_le_bytes(data[3..7].try_into().unwrap());
            let target = offset as i64 + 7 + displacement as i64;
            instructions.push(format!("0x{offset:X}: lea {register},[rip{displacement:+}] -> 0x{target:X}"));
            index += 7;
            continue;
        }
        if data.len() >= 4 && data[0] == 0x48 && data[1] == 0x8d && data[2] == 0x4d {
            instructions.push(format!(
                "0x{offset:X}: lea rcx,[rbp{}]",
                data[3] as i8
            ));
            index += 4;
            continue;
        }
        if data.len() >= 6 && data[0] == 0x41 && data[1] == 0xb8 {
            let immediate = u32::from_le_bytes(data[2..6].try_into().unwrap());
            instructions.push(format!("0x{offset:X}: mov r8d,0x{immediate:X}"));
            index += 6;
            continue;
        }
        if data.len() >= 5 && data[0] == 0xe8 {
            let displacement = i32::from_le_bytes(data[1..5].try_into().unwrap());
            let target = offset as i64 + 5 + displacement as i64;
            instructions.push(format!("0x{offset:X}: call rel32 -> 0x{target:X}"));
            index += 5;
            continue;
        }
        if data.len() >= 6 && data[0] == 0xff && data[1] == 0x15 {
            let displacement = i32::from_le_bytes(data[2..6].try_into().unwrap());
            let target = offset as i64 + 6 + displacement as i64;
            instructions.push(format!("0x{offset:X}: call qword [rip{displacement:+}] -> 0x{target:X}"));
            index += 6;
            continue;
        }
        if data.len() >= 5 && data[0] == 0xe9 {
            let displacement = i32::from_le_bytes(data[1..5].try_into().unwrap());
            let target = offset as i64 + 5 + displacement as i64;
            instructions.push(format!("0x{offset:X}: jmp rel32 -> 0x{target:X}"));
            index += 5;
            continue;
        }
        if data.len() >= 2 && data[0] == 0xeb {
            let target = offset as i64 + 2 + data[1] as i8 as i64;
            instructions.push(format!("0x{offset:X}: jmp short -> 0x{target:X}"));
            index += 2;
            continue;
        }
        if is_conditional_jump(&reference.context_bytes, index, reference.context_bytes.len()) {
            if (0x70..=0x7f).contains(&data[0]) && data.len() >= 2 {
                let target = offset as i64 + 2 + data[1] as i8 as i64;
                instructions.push(format!("0x{offset:X}: jcc short 0x{:02X} -> 0x{target:X}", data[0]));
                index += 2;
                continue;
            }
            if data.len() >= 6 && data[0] == 0x0f {
                let displacement = i32::from_le_bytes(data[2..6].try_into().unwrap());
                let target = offset as i64 + 6 + displacement as i64;
                instructions.push(format!("0x{offset:X}: jcc near 0x{:02X} -> 0x{target:X}", data[1]));
                index += 6;
                continue;
            }
        }
        if data.len() >= 4 && data[0] == 0x48 && data[1] == 0x83 && (data[2] == 0xec || data[2] == 0xc4) {
            let op = if data[2] == 0xc4 { "add" } else { "sub" };
            instructions.push(format!("0x{offset:X}: {op} rsp,0x{:X}", data[3]));
            index += 4;
            continue;
        }
        index += 1;
    }
    if instructions.is_empty() {
        return "none".to_string();
    }
    if instructions.len() > 10 {
        return format!("{}; ... +{} more", instructions[..10].join("; "), instructions.len() - 10);
    }
    instructions.join("; ")
}

pub fn log_battleye_signature_report(log: &mut LogSink, report: &DiagnosisReport) {
    log.info("Known BattlEye byte patch signature report:");
    for status in &report.patch_statuses {
        let signature_kind = if status.diagnostic_only {
            "diagnostic-only"
        } else {
            "patchable"
        };
        if !status.original_offset.is_empty() {
            log.warn(format!(
                "{:?} ({signature_kind}) original signature present at {}",
                status.name,
                format_offsets_limited(&status.original_offset, 0)
            ));
        } else if !status.patched_offset.is_empty() {
            log.info(format!(
                "{:?} ({signature_kind}) patched signature present at {}",
                status.name,
                format_offsets_limited(&status.patched_offset, 0)
            ));
        } else {
            log.info(format!("{:?} ({signature_kind}) signature not found", status.name));
        }
        for expected in &status.expected_offset_hits {
            log.info(format!(
                "  expected offset hit 0x{:X} for SHA256 {}: {}",
                expected.offset, expected.sha256, expected.note
            ));
        }
        for expected in &status.expected_offset_misses {
            log.warn(format!(
                "  expected offset miss 0x{:X} for SHA256 {}: {}",
                expected.offset, expected.sha256, expected.note
            ));
        }
        if status.diagnostic_only && !status.false_positive_check.is_empty() {
            log.info(format!("  aob mask: {}", status.aob_mask));
            log.info(format!("  false-positive guard: {}", status.false_positive_check));
        }
    }
}

pub fn patch_state_by_name(report: &DiagnosisReport, name: &str) -> String {
    for status in &report.patch_statuses {
        if status.name != name {
            continue;
        }
        if !status.original_offset.is_empty() && !status.patched_offset.is_empty() {
            return format!(
                "mixed original={} patched={}",
                format_offsets_limited(&status.original_offset, 4),
                format_offsets_limited(&status.patched_offset, 4)
            );
        }
        if !status.original_offset.is_empty() {
            return format!("original at {}", format_offsets_limited(&status.original_offset, 4));
        }
        if !status.patched_offset.is_empty() {
            return format!("patched at {}", format_offsets_limited(&status.patched_offset, 4));
        }
        return "absent".to_string();
    }
    "unknown".to_string()
}

pub fn client_check_indicator_keys(report: &DiagnosisReport) -> Vec<String> {
    let mut keys: Vec<String> = report
        .client_check_findings
        .iter()
        .map(|f| format!("{}/{}", f.name, f.encoding))
        .collect();
    keys.sort();
    keys
}

pub fn client_check_indicator_count(report: &DiagnosisReport) -> usize {
    report.client_check_findings.iter().map(|f| f.offsets.len()).sum()
}

pub fn client_check_code_reference_count(report: &DiagnosisReport) -> usize {
    report.client_check_findings.iter().map(|f| f.references.len()).sum()
}

pub fn suspicious_active_indicator_keys(report: &DiagnosisReport) -> Vec<String> {
    let mut key_set = HashSet::new();
    for finding in &report.client_check_findings {
        for reference in &finding.references {
            if reference.suspicious_active {
                key_set.insert(format!("{}/{}", finding.name, finding.encoding));
            }
        }
    }
    let mut keys: Vec<String> = key_set.into_iter().collect();
    keys.sort();
    keys
}

pub fn strong_unsupported_evidence_keys(report: &DiagnosisReport) -> Vec<String> {
    let mut keys = Vec::new();
    for finding in &report.client_check_findings {
        for reference in &finding.references {
            if !reference.strong_unsupported {
                continue;
            }
            keys.push(format!(
                "{}/{} ref=0x{:X} branches={} calls={}",
                finding.name,
                finding.encoding,
                reference.offset,
                format_offsets_limited(&reference.branch_offsets, 3),
                format_offsets_limited(&reference.call_offsets, 3),
            ));
        }
    }
    keys.sort();
    keys
}

pub fn suspicious_active_evidence_keys(report: &DiagnosisReport) -> Vec<String> {
    let mut keys = Vec::new();
    for finding in &report.client_check_findings {
        for reference in &finding.references {
            if !reference.suspicious_active {
                continue;
            }
            keys.push(format!(
                "{}/{} ref=0x{:X} branches={} calls={}",
                finding.name,
                finding.encoding,
                reference.offset,
                format_nearest_offsets(reference.offset, &reference.branch_offsets, 3),
                format_nearest_offsets(reference.offset, &reference.call_offsets, 3),
            ));
        }
    }
    keys.sort();
    keys
}

// Re-export for battleye module
pub use super::patterns::new_byte_pattern;
