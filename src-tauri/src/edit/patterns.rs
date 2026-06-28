pub const WILDCARD_BYTE: i32 = -1;

pub const CODE_CONTEXT_RADIUS: usize = 200;
pub const CONTEXT_BYTES_RADIUS: usize = 48;
pub const KNOWN_PATCH_CONTEXT_RADIUS: usize = 512;
pub const PATCH_CONTEXT_RADIUS: usize = 32;

#[derive(Debug, Clone)]
pub struct BytePattern {
    pub name: String,
    pub data: Vec<u8>,
    pub mask: Vec<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct KnownPatchOffsetRaw {
    pub sha256: &'static str,
    pub offset: usize,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub struct BattleyePatch {
    pub name: &'static str,
    pub original: BytePattern,
    pub patched: BytePattern,
    pub replacement: Vec<i32>,
    pub diagnostic_only: bool,
    pub aggressive_replacement: Vec<i32>,
    pub high_risk_client_check: bool,
    pub expected_offsets: Vec<KnownPatchOffsetRaw>,
    pub false_positive_check: &'static str,
}

pub fn new_byte_pattern(name: &str, values: &[i32]) -> BytePattern {
    let mut pattern = BytePattern {
        name: name.to_string(),
        data: vec![0; values.len()],
        mask: vec![false; values.len()],
    };
    for (index, &value) in values.iter().enumerate() {
        if value == WILDCARD_BYTE {
            continue;
        }
        assert!(
            (0..=0xff).contains(&value),
            "invalid byte pattern value {value} in {name}"
        );
        pattern.data[index] = value as u8;
        pattern.mask[index] = true;
    }
    pattern
}

pub fn new_patch_replacement(values: &[i32]) -> Vec<i32> {
    values.to_vec()
}

pub fn neutralize_branch_jump_pattern(pattern: &BytePattern, patch: &[(usize, u8)]) -> Vec<i32> {
    let mut replacement: Vec<i32> = pattern
        .mask
        .iter()
        .enumerate()
        .map(|(index, &masked)| {
            if !masked {
                WILDCARD_BYTE
            } else {
                pattern.data[index] as i32
            }
        })
        .collect();
    for &(index, value) in patch {
        assert!(index < replacement.len());
        replacement[index] = value as i32;
    }
    replacement
}

impl BytePattern {
    pub fn format_aob(&self) -> String {
        if self.data.is_empty() {
            return "none".to_string();
        }
        self.data
            .iter()
            .zip(self.mask.iter())
            .map(|(&value, &masked)| {
                if masked {
                    format!("{value:02X}")
                } else {
                    "??".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn find_all(&self, data: &[u8]) -> Vec<usize> {
        let mut offsets = Vec::new();
        if self.data.is_empty() || data.len() < self.data.len() {
            return offsets;
        }
        for offset in 0..=data.len() - self.data.len() {
            if self.matches_at(data, offset) {
                offsets.push(offset);
            }
        }
        offsets
    }

    pub fn matches_at(&self, data: &[u8], offset: usize) -> bool {
        if offset + self.data.len() > data.len() {
            return false;
        }
        for (index, &masked) in self.mask.iter().enumerate() {
            if masked && data[offset + index] != self.data[index] {
                return false;
            }
        }
        true
    }
}

impl BattleyePatch {
    pub fn with_aggressive_mode(&self, aggressive: bool) -> BattleyePatch {
        let mut patch = self.clone();
        if !aggressive || patch.aggressive_replacement.is_empty() {
            return patch;
        }
        assert_eq!(
            patch.aggressive_replacement.len(),
            patch.original.data.len(),
            "invalid aggressive replacement for signature {}",
            patch.name
        );
        patch.replacement = patch.aggressive_replacement.clone();
        patch.patched = new_byte_pattern(
            &format!("{} [aggressive]", patch.name),
            &patch.aggressive_replacement,
        );
        patch
    }

    pub fn expected_offset_hits(&self, data: &[u8], sha256_text: &str) -> Vec<KnownPatchOffsetRaw> {
        self.expected_offsets
            .iter()
            .filter(|e| e.applies_to_sha256(sha256_text))
            .filter(|e| self.matches_at_expected_offset(data, e.offset))
            .copied()
            .collect()
    }

    pub fn expected_offset_misses(
        &self,
        data: &[u8],
        sha256_text: &str,
    ) -> Vec<KnownPatchOffsetRaw> {
        self.expected_offsets
            .iter()
            .filter(|e| e.applies_to_sha256(sha256_text))
            .filter(|e| !self.matches_at_expected_offset(data, e.offset))
            .copied()
            .collect()
    }

    fn matches_at_expected_offset(&self, data: &[u8], offset: usize) -> bool {
        self.original.matches_at(data, offset) || self.patched.matches_at(data, offset)
    }
}

impl KnownPatchOffsetRaw {
    pub fn applies_to_sha256(&self, sha256_text: &str) -> bool {
        self.sha256.is_empty() || self.sha256.eq_ignore_ascii_case(sha256_text)
    }
}

pub fn battleye_patches() -> Vec<BattleyePatch> {
    vec![
        BattleyePatch {
            name: "legacy launch check",
            original: new_byte_pattern(
                "legacy launch check original",
                &[0x8d, 0x4d, 0xb4, 0x75, 0x0e, 0xe8, 0xb4, 0x53],
            ),
            patched: new_byte_pattern(
                "legacy launch check patched",
                &[0x8d, 0x4d, 0xb4, 0xeb, 0x0e, 0xe8, 0xb4, 0x53],
            ),
            replacement: new_patch_replacement(&[
                0x8d, 0x4d, 0xb4, 0xeb, 0x0e, 0xe8, 0xb4, 0x53,
            ]),
            diagnostic_only: false,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![],
            false_positive_check: "",
        },
        BattleyePatch {
            name: "client check branch",
            original: new_byte_pattern(
                "client check branch original",
                &[0x75, 0x0f, 0xe8, 0x35, 0xff, 0xff, 0xff, 0x48],
            ),
            patched: new_byte_pattern(
                "client check branch patched",
                &[0xeb, 0x0f, 0xe8, 0x35, 0xff, 0xff, 0xff, 0x48],
            ),
            replacement: new_patch_replacement(&[
                0xeb, 0x0f, 0xe8, 0x35, 0xff, 0xff, 0xff, 0x48,
            ]),
            diagnostic_only: false,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![KnownPatchOffsetRaw {
                sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                offset: 0x2DE804,
                note: "reported new client; this is the only currently observed matching legacy patch point",
            }],
            false_positive_check: "",
        },
        BattleyePatch {
            name: "legacy client check branch",
            original: new_byte_pattern(
                "legacy client check branch original",
                &[0x75, 0x0f, 0xe8, 0xd9, 0xd4, 0xed, 0xff, 0x48],
            ),
            patched: new_byte_pattern(
                "legacy client check branch patched",
                &[0xeb, 0x0f, 0xe8, 0xd9, 0xd4, 0xed, 0xff, 0x48],
            ),
            replacement: new_patch_replacement(&[
                0xeb, 0x0f, 0xe8, 0xd9, 0xd4, 0xed, 0xff, 0x48,
            ]),
            diagnostic_only: false,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![],
            false_positive_check: "",
        },
        BattleyePatch {
            name: "candidate client check conditional branch with variable call",
            original: new_byte_pattern(
                "candidate client check conditional branch original",
                &[
                    0x75, 0x0f, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    0x48,
                ],
            ),
            patched: new_byte_pattern(
                "candidate client check conditional branch patched",
                &[
                    0xeb, 0x0f, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    0x48,
                ],
            ),
            replacement: new_patch_replacement(&[
                0xeb, 0x0f, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, 0x48,
            ]),
            diagnostic_only: true,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![KnownPatchOffsetRaw {
                sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                offset: 0x2DE804,
                note: "wildcard diagnostic around the reported new-client match; not auto-applied without surrounding-code review",
            }],
            false_positive_check: "diagnostic-only because the CALL rel32 bytes are wildcarded; require unique match, nearby client-check xref, and manual code-context review before making this patchable",
        },
        BattleyePatch {
            name: "candidate clientcheck_disconnected Qt xref dispatch",
            original: new_byte_pattern(
                "candidate clientcheck_disconnected Qt xref dispatch",
                &[
                    0x41, 0xb8, 0xff, 0xff, 0xff, 0xff, 0x48, 0x8d, 0x15, 0x18, 0x39, 0x80, 0x01,
                    0x48, 0x8d, 0x4d, 0x37, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE,
                ],
            ),
            patched: new_byte_pattern("candidate clientcheck_disconnected Qt xref dispatch patched", &[]),
            replacement: vec![],
            diagnostic_only: true,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![
                KnownPatchOffsetRaw {
                    sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                    offset: 0x1A8E5B,
                    note: "reported new-client clientcheck_disconnected xref context",
                },
                KnownPatchOffsetRaw {
                    sha256: "985fb4e114b3156a5488b7b35ed5d8615d58fff140a04d8e73c18ac0b4d871e5",
                    offset: 0x1A8E5B,
                    note: "observed local clientcheck_disconnected xref context",
                },
            ],
            false_positive_check: "diagnostic-only xref context observed around reported ref 0x1A8E61; exact displacement bytes keep this version-specific until the RIP target and branch/call flow are manually reviewed",
        },
        BattleyePatch {
            name: "candidate BEClient Qt xref dispatch",
            original: new_byte_pattern(
                "candidate BEClient Qt xref dispatch",
                &[
                    0x48, 0x8b, 0x01, 0x48, 0x8b, 0x58, 0x28, 0x48, 0x8d, 0x15, 0x45, 0x1a, 0x7f,
                    0x01, 0x48, 0x8d, 0x4c, 0x24, 0x28, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE,
                ],
            ),
            patched: new_byte_pattern("candidate BEClient Qt xref dispatch patched", &[]),
            replacement: vec![],
            diagnostic_only: true,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![
                KnownPatchOffsetRaw {
                    sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                    offset: 0x1BB425,
                    note: "reported new-client BEClient xref context",
                },
                KnownPatchOffsetRaw {
                    sha256: "985fb4e114b3156a5488b7b35ed5d8615d58fff140a04d8e73c18ac0b4d871e5",
                    offset: 0x1BB425,
                    note: "observed local BEClient xref context",
                },
            ],
            false_positive_check: "diagnostic-only xref context observed around reported ref 0x1BB42C; BEClient remains weak by itself because Qt metadata/dialog text can reference it without proving active client-check flow",
        },
        BattleyePatch {
            name: "high-risk clientcheck_disconnected dispatch path",
            original: new_byte_pattern(
                "high-risk clientcheck_disconnected dispatch path",
                &[
                    0x48, 0x83, 0x45, 0x9f, 0x48, 0xeb, 0x10, 0x4c, 0x8d, 0x45, 0xb7, 0x48, 0x8b,
                    0xd3, 0x48, 0x8d, 0x4d, 0x97, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, 0x48, 0x8b, 0xbf, 0x30, 0x0a, 0x00, 0x00, 0x41, 0xb8, 0xff, 0xff,
                    0xff, 0xff, 0x48, 0x8d, 0x15, 0x18, 0x39, 0x80, 0x01, 0x48, 0x8d, 0x4d, 0x37,
                    0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, 0x48,
                    0x8b, 0xd8, 0x41, 0xb8, 0xff, 0xff, 0xff, 0xff, 0x48, 0x8d, 0x15, 0xbe, 0x4a,
                    0x7d, 0x01, 0x48, 0x8d, 0x4d, 0x1f, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE, 0x90, 0x4c, 0x8d, 0x4d, 0x97, 0x4c, 0x8b, 0xc3,
                    0x48, 0x8b, 0xd0, 0x48, 0x8b, 0xcf, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE, 0x90,
                ],
            ),
            patched: new_byte_pattern("high-risk clientcheck_disconnected dispatch path patched", &[]),
            replacement: vec![],
            diagnostic_only: true,
            aggressive_replacement: neutralize_branch_jump_pattern(
                &new_byte_pattern(
                    "high-risk clientcheck_disconnected dispatch path [aggressive source]",
                    &[
                        0x48, 0x83, 0x45, 0x9f, 0x48, 0xeb, 0x10, 0x4c, 0x8d, 0x45, 0xb7, 0x48, 0x8b,
                        0xd3, 0x48, 0x8d, 0x4d, 0x97, 0xe8, WILDCARD_BYTE, WILDCARD_BYTE,
                        WILDCARD_BYTE, WILDCARD_BYTE, 0x48, 0x8b, 0xbf, 0x30, 0x0a, 0x00, 0x00, 0x41,
                        0xb8, 0xff, 0xff, 0xff, 0xff, 0x48, 0x8d, 0x15, 0x18, 0x39, 0x80, 0x01,
                        0x48, 0x8d, 0x4d, 0x37, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                        WILDCARD_BYTE, WILDCARD_BYTE, 0x48, 0x8b, 0xd8, 0x41, 0xb8, 0xff, 0xff, 0xff,
                        0xff, 0x48, 0x8d, 0x15, 0xbe, 0x4a, 0x7d, 0x01, 0x48, 0x8d, 0x4d, 0x1f, 0xff,
                        0x15, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, 0x90, 0x4c,
                        0x8d, 0x4d, 0x97, 0x4c, 0x8b, 0xc3, 0x48, 0x8b, 0xd0, 0x48, 0x8b, 0xcf, 0xe8,
                        WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, 0x90,
                    ],
                ),
                &[(93, 0x90), (94, 0x90), (95, 0x90), (96, 0x90), (97, 0x90)],
            ),
            high_risk_client_check: true,
            expected_offsets: vec![
                KnownPatchOffsetRaw {
                    sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                    offset: 0x1A8E3D,
                    note: "high-risk clientcheck_disconnected dispatch path seen after the known 0x2DE804 patch",
                },
                KnownPatchOffsetRaw {
                    sha256: "985fb4e114b3156a5488b7b35ed5d8615d58fff140a04d8e73c18ac0b4d871e5",
                    offset: 0x1A8E3D,
                    note: "observed local clientcheck_disconnected dispatch path seen after the known 0x2DE804 patch",
                },
            ],
            false_positive_check: "diagnostic-only high-risk path; CALL bytes are wildcarded, but fixed surrounding xref/field-access bytes tie it to the reported clientcheck_disconnected dispatch context; aggressive mode nops the final signal dispatch call",
        },
        BattleyePatch {
            name: "candidate enableClientCheck Qt xref dispatch",
            original: new_byte_pattern(
                "candidate enableClientCheck Qt xref dispatch",
                &[
                    0x48, 0x83, 0xec, 0x28, 0x48, 0x8d, 0x15, 0x65, 0xc5, 0x99, 0x01, 0x48, 0x8d,
                    0x0d, 0x36, 0x04, 0xcf, 0x01, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE, 0x48, 0x8d, 0x0d, 0x11, 0xd0, 0xf7, 0x00,
                ],
            ),
            patched: new_byte_pattern("candidate enableClientCheck Qt xref dispatch patched", &[]),
            replacement: vec![],
            diagnostic_only: true,
            aggressive_replacement: vec![],
            high_risk_client_check: false,
            expected_offsets: vec![
                KnownPatchOffsetRaw {
                    sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                    offset: 0xE8C0,
                    note: "reported new-client enableClientCheck xref context",
                },
                KnownPatchOffsetRaw {
                    sha256: "985fb4e114b3156a5488b7b35ed5d8615d58fff140a04d8e73c18ac0b4d871e5",
                    offset: 0xE8C0,
                    note: "observed local enableClientCheck xref context",
                },
            ],
            false_positive_check: "diagnostic-only xref context observed around reported ref 0xE8C4; exact displacement bytes keep this version-specific until the RIP target and branch/call flow are manually reviewed",
        },
        BattleyePatch {
            name: "high-risk enableClientCheck dispatch path",
            original: new_byte_pattern(
                "high-risk enableClientCheck dispatch path",
                &[
                    0x48, 0x83, 0xec, 0x28, 0x48, 0x8d, 0x15, 0x65, 0xc5, 0x99, 0x01, 0x48, 0x8d,
                    0x0d, 0x36, 0x04, 0xcf, 0x01, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE, 0x48, 0x8d, 0x0d, 0x11, 0xd0, 0xf7, 0x00, 0x48,
                    0x83, 0xc4, 0x28, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                ],
            ),
            patched: new_byte_pattern("high-risk enableClientCheck dispatch path patched", &[]),
            replacement: vec![],
            diagnostic_only: true,
            aggressive_replacement: neutralize_branch_jump_pattern(
                &new_byte_pattern(
                    "high-risk enableClientCheck dispatch path [aggressive source]",
                    &[
                        0x48, 0x83, 0xec, 0x28, 0x48, 0x8d, 0x15, 0x65, 0xc5, 0x99, 0x01, 0x48,
                        0x8d, 0x0d, 0x36, 0x04, 0xcf, 0x01, 0xff, 0x15, WILDCARD_BYTE, WILDCARD_BYTE,
                        WILDCARD_BYTE, WILDCARD_BYTE, 0x48, 0x8d, 0x0d, 0x11, 0xd0, 0xf7, 0x00, 0x48,
                        0x83, 0xc4, 0x28, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                        WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE, WILDCARD_BYTE,
                    ],
                ),
                &[
                    (18, 0x90),
                    (19, 0x90),
                    (20, 0x90),
                    (21, 0x90),
                    (22, 0x90),
                    (23, 0x90),
                ],
            ),
            high_risk_client_check: true,
            expected_offsets: vec![
                KnownPatchOffsetRaw {
                    sha256: "c930bd29b76cec5d88d35e24dbee0ed0edaeba68bd7961c68856912c40d8728f",
                    offset: 0xE8C0,
                    note: "high-risk enableClientCheck dispatch path seen after the known 0x2DE804 patch",
                },
                KnownPatchOffsetRaw {
                    sha256: "985fb4e114b3156a5488b7b35ed5d8615d58fff140a04d8e73c18ac0b4d871e5",
                    offset: 0xE8C0,
                    note: "observed local enableClientCheck dispatch path seen after the known 0x2DE804 patch",
                },
            ],
            false_positive_check: "diagnostic-only high-risk path; CALL/JMP bytes are wildcarded, but fixed enableClientCheck xref and thunk shape keep the match scoped to the reported dispatch context; aggressive mode nops only the Qt metadata call and preserves the original tail jump",
        },
    ]
}

pub struct ClientCheckIndicator {
    pub name: &'static str,
    pub value: &'static [u8],
}

pub fn client_check_indicators() -> Vec<ClientCheckIndicator> {
    vec![
        ClientCheckIndicator {
            name: "BEClient",
            value: b"BEClient",
        },
        ClientCheckIndicator {
            name: "clientcheck_disconnected",
            value: b"clientcheck_disconnected",
        },
        ClientCheckIndicator {
            name: "requestCloseDueToClientCheck",
            value: b"requestCloseDueToClientCheck",
        },
        ClientCheckIndicator {
            name: "onCloseDueToClientCheckRequested",
            value: b"onCloseDueToClientCheckRequested",
        },
        ClientCheckIndicator {
            name: "onClientCheckDialogButtonClicked",
            value: b"onClientCheckDialogButtonClicked",
        },
        ClientCheckIndicator {
            name: "enableClientCheck",
            value: b"enableClientCheck",
        },
    ]
}

pub fn client_check_code_patterns() -> Vec<BytePattern> {
    vec![
        new_byte_pattern(
            "short JNE followed by CALL",
            &[
                0x75,
                WILDCARD_BYTE,
                0xe8,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
            ],
        ),
        new_byte_pattern(
            "short JE followed by CALL",
            &[
                0x74,
                WILDCARD_BYTE,
                0xe8,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
            ],
        ),
        new_byte_pattern(
            "near JNE followed by CALL",
            &[
                0x0f,
                0x85,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                0xe8,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
            ],
        ),
        new_byte_pattern(
            "near JE followed by CALL",
            &[
                0x0f,
                0x84,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                0xe8,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
                WILDCARD_BYTE,
            ],
        ),
    ]
}

pub const QT_CONTEXT_INDICATORS: &[&str] = &[
    "Qt5Core",
    "Qt6Core",
    "QMetaObject",
    "QObject",
    "qt_metacall",
    "qt_static_metacall",
    "QMessageBox",
];

pub fn patchable_battleye_patch_count() -> usize {
    battleye_patches()
        .iter()
        .filter(|p| !p.diagnostic_only)
        .count()
}

pub const URL_PROPERTIES: &[&str] = &[
    "loginWebService",
    "clientWebService",
    "tibiaPageUrl",
    "tibiaStoreGetCoinsUrl",
    "getPremiumUrl",
    "createAccountUrl",
    "accessAccountUrl",
    "lostAccountUrl",
    "manualUrl",
    "faqUrl",
    "premiumFeaturesUrl",
    "crashReportUrl",
    "fpsHistoryRecipient",
    "cipSoftUrl",
];
