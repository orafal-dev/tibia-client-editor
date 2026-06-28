export type DiagnosisReport = {
  path: string;
  size: number;
  sha256: string;
  is_windows_exe: boolean;
  pe: PeInfo;
  patch_statuses: BattleyePatchStatus[];
  client_check_findings: ClientCheckFinding[];
  qt_indicators: string[];
  client_check_verdict: string;
  known_patch_coverage: number;
  patchable_count: number;
  original_patch_signature_count: number;
  patched_patch_signature_count: number;
  strong_unsupported_evidence_count: number;
  suspicious_active_evidence_count: number;
  high_risk_diagnostic_count: number;
};

export type PeInfo = {
  valid: boolean;
  error_text?: string;
  image_base: number;
  sections: PeSectionInfo[];
  imports: string[];
};

export type PeSectionInfo = {
  name: string;
  raw_start: number;
  raw_end: number;
  rva_start: number;
  rva_end: number;
  is_code: boolean;
};

export type BattleyePatchStatus = {
  name: string;
  diagnostic_only: boolean;
  high_risk_client_check: boolean;
  false_positive_check: string;
  original_offset: number[];
  patched_offset: number[];
  expected_offset_hits: KnownPatchOffset[];
  expected_offset_misses: KnownPatchOffset[];
  aob_mask: string;
};

export type KnownPatchOffset = {
  sha256: string;
  offset: number;
  note: string;
};

export type ClientCheckFinding = {
  name: string;
  encoding: string;
  offsets: number[];
  references: ClientCheckReference[];
};

export type ClientCheckReference = {
  offset: number;
  section: string;
  instruction: string;
  branch_offsets: number[];
  call_offsets: number[];
  pattern_matches: PatternMatch[];
  context_start: number;
  context_bytes: string;
  known_patch_nearby: boolean;
  strong_unsupported: boolean;
  suspicious_active: boolean;
  possible_instructions: string;
};

export type PatternMatch = {
  name: string;
  offset: number;
};

export type EditOutput = {
  logs: string[];
  diagnosis: DiagnosisReport;
  success: boolean;
  output_path: string;
  backup_path?: string;
  properties_patched: string[];
  properties_failed: string[];
};

export type DiagnoseOutput = {
  logs: string[];
  target: DiagnosisReport;
  baseline?: DiagnosisReport;
  comparison_logs?: string[];
};

export type RepackResult = {
  logs: string[];
  client_files: number;
  assets_files: number;
  revision: number;
  output_dir: string;
};

export type Win2MacResult = {
  logs: string[];
  output_path: string;
  files_updated: number;
};

export type AppearancesResult = {
  logs: string[];
  edits_applied: number;
  output_path: string;
};

export type ClientConfig = {
  loginWebService: string;
  clientWebService: string;
  tibiaPageUrl: string;
  tibiaStoreGetCoinsUrl: string;
  getPremiumUrl: string;
  createAccountUrl: string;
  accessAccountUrl: string;
  lostAccountUrl: string;
  manualUrl: string;
  faqUrl: string;
  premiumFeaturesUrl: string;
  crashReportUrl: string;
  fpsHistoryRecipient: string;
  cipSoftUrl: string;
};

export const URL_PROPERTY_KEYS: (keyof ClientConfig)[] = [
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

export const URL_PROPERTY_LABELS: Record<keyof ClientConfig, string> = {
  loginWebService: "Login Web Service",
  clientWebService: "Client Web Service",
  tibiaPageUrl: "Tibia Page URL",
  tibiaStoreGetCoinsUrl: "Store Coins URL",
  getPremiumUrl: "Premium URL",
  createAccountUrl: "Create Account URL",
  accessAccountUrl: "Access Account URL",
  lostAccountUrl: "Lost Account URL",
  manualUrl: "Manual URL",
  faqUrl: "FAQ URL",
  premiumFeaturesUrl: "Premium Features URL",
  crashReportUrl: "Crash Report URL",
  fpsHistoryRecipient: "FPS History Recipient",
  cipSoftUrl: "CipSoft URL",
};

export const DEFAULT_LOCAL_CONFIG: ClientConfig = {
  loginWebService: "http://127.0.0.1:7171/api/login",
  clientWebService: "http://127.0.0.1:7171/api/login",
  tibiaPageUrl: "http://127.0.0.1:7171/",
  tibiaStoreGetCoinsUrl: "http://127.0.0.1:7171/shop/coins",
  getPremiumUrl: "http://127.0.0.1:7171/pages/vip-features",
  createAccountUrl: "http://127.0.0.1:7171/account/signup",
  accessAccountUrl: "http://127.0.0.1:7171/account",
  lostAccountUrl: "http://127.0.0.1:7171/account/lost",
  manualUrl: "http://127.0.0.1:7171/pages/server-info",
  faqUrl: "http://127.0.0.1:7171/pages/server-info",
  premiumFeaturesUrl: "http://127.0.0.1:7171/pages/vip-features",
  crashReportUrl: "http://127.0.0.1:7171/api/crash-report",
  cipSoftUrl: "http://127.0.0.1:7171/",
  fpsHistoryRecipient: "http://127.0.0.1:7171/api/hardware-report",
};

export type RepackPlatform = "windows" | "mac" | "linux";
