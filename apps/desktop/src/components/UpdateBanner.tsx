import { Download, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { UpdateCheck, UpdateProgress } from "@/types";

interface UpdateBannerProps {
  update: UpdateCheck | null;
  checking?: boolean;
  installing?: boolean;
  progress?: UpdateProgress | null;
  onCheck: () => void;
  onInstall: () => void;
  onOpenRelease: () => void;
}

export function UpdateBanner({
  update,
  checking,
  installing,
  progress,
  onCheck,
  onInstall,
  onOpenRelease,
}: UpdateBannerProps) {
  if (!update?.available) {
    return null;
  }

  const percent =
    progress?.total && progress.total > 0
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;

  return (
    <div className="border-primary/30 bg-primary/10 flex flex-col gap-2 rounded-xl border px-3 py-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="min-w-0">
          <p className="text-sm font-medium">Update available: v{update.latest_version}</p>
          <p className="text-muted-foreground text-xs">
            {update.can_install_in_app
              ? `You are on v${update.current_version}. KnotTrace can install this update and restart.`
              : `You are on v${update.current_version}. Download the installer from GitHub.`}
          </p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" disabled={checking || installing} onClick={onCheck}>
            {checking ? "Checking..." : "Check again"}
          </Button>
          {update.can_install_in_app ? (
            <Button size="sm" disabled={installing} onClick={onInstall}>
              {installing ? (
                <>
                  <Loader2 className="size-4 animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <Download className="size-4" />
                  Install update
                </>
              )}
            </Button>
          ) : (
            <Button size="sm" onClick={onOpenRelease}>
              <Download className="size-4" />
              Get update
            </Button>
          )}
        </div>
      </div>
      {installing && (
        <div className="space-y-1">
          <div className="bg-background/50 h-1.5 overflow-hidden rounded-full">
            <div
              className="bg-primary h-full transition-all"
              style={{ width: `${percent ?? (progress?.phase === "finished" ? 100 : 12)}%` }}
            />
          </div>
          <p className="text-muted-foreground text-[0.68rem]">
            {progress?.phase === "finished"
              ? "Restarting KnotTrace..."
              : progress?.phase === "progress" && percent != null
                ? `Downloading ${percent}%`
                : "Preparing update..."}
          </p>
        </div>
      )}
    </div>
  );
}
