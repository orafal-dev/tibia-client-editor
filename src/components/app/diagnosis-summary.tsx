import type { DiagnosisReport } from "@/lib/models.types";
import { Badge } from "@/components/ui/badge";

type VerdictBadgeProps = {
  verdict: string;
};

const verdictVariant = (verdict: string) => {
  switch (verdict.toUpperCase()) {
    case "SUPPORTED":
      return "success" as const;
    case "PARTIAL":
      return "warning" as const;
    case "WARNING":
      return "warning" as const;
    case "UNSUPPORTED":
      return "destructive" as const;
    default:
      return "secondary" as const;
  }
};

export const VerdictBadge = ({ verdict }: VerdictBadgeProps) => (
  <Badge variant={verdictVariant(verdict)}>{verdict}</Badge>
);

type DiagnosisSummaryProps = {
  report: DiagnosisReport;
  title?: string;
};

export const DiagnosisSummary = ({ report, title = "Diagnosis" }: DiagnosisSummaryProps) => (
  <div className="grid gap-3 rounded-lg border p-4">
    <div className="flex flex-wrap items-center justify-between gap-2">
      <h3 className="font-medium">{title}</h3>
      <VerdictBadge verdict={report.client_check_verdict} />
    </div>
    <dl className="grid gap-2 text-sm sm:grid-cols-2">
      <div>
        <dt className="text-muted-foreground">File</dt>
        <dd className="truncate font-mono text-xs">{report.path}</dd>
      </div>
      <div>
        <dt className="text-muted-foreground">SHA256</dt>
        <dd className="truncate font-mono text-xs">{report.sha256}</dd>
      </div>
      <div>
        <dt className="text-muted-foreground">Size</dt>
        <dd>{report.size.toLocaleString()} bytes</dd>
      </div>
      <div>
        <dt className="text-muted-foreground">Patch coverage</dt>
        <dd>
          {report.known_patch_coverage}/{report.patchable_count}
        </dd>
      </div>
      <div>
        <dt className="text-muted-foreground">Strong unsupported</dt>
        <dd>{report.strong_unsupported_evidence_count}</dd>
      </div>
      <div>
        <dt className="text-muted-foreground">Suspicious active</dt>
        <dd>{report.suspicious_active_evidence_count}</dd>
      </div>
    </dl>
  </div>
);
