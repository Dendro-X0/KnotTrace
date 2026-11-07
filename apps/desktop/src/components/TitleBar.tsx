import { useEffect, useState, type ReactNode } from "react";
import { Minus, Square, X } from "lucide-react";

import { useLocale } from "@/hooks/useLocale";
import { cn } from "@/lib/utils";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function withWindow(
  run: (win: {
    minimize: () => Promise<void>;
    toggleMaximize: () => Promise<void>;
    close: () => Promise<void>;
    isMaximized: () => Promise<boolean>;
    onResized: (handler: () => void) => Promise<() => void>;
  }) => void,
): Promise<void> {
  if (!isTauriRuntime()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await run(getCurrentWindow());
}

export function TitleBar() {
  const { t } = useLocale();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      await withWindow(async (win) => {
        const sync = async () => {
          const next = await win.isMaximized();
          if (!cancelled) setMaximized(next);
        };
        await sync();
        unlisten = await win.onResized(() => {
          void sync();
        });
      });
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return (
    <header
      className={cn(
        "relative flex h-10 shrink-0 select-none items-stretch",
        "border-b border-border/40 bg-transparent text-foreground",
        "after:pointer-events-none after:absolute after:inset-x-0 after:bottom-0 after:h-px",
        "after:bg-gradient-to-r after:from-transparent after:via-primary/35 after:to-transparent",
      )}
      aria-label={t("chrome.dragHint")}
    >
      <div
        className="flex min-w-0 flex-1 items-center gap-2.5 px-3.5"
        data-tauri-drag-region
        onDoubleClick={() => {
          void withWindow((win) => win.toggleMaximize());
        }}
      >
        <img
          src="/knottrace-icon.png"
          alt=""
          className="pointer-events-none size-5 shrink-0 rounded-md shadow-[0_0_12px_-2px] shadow-primary/40"
          data-tauri-drag-region
        />
        <div className="pointer-events-none min-w-0" data-tauri-drag-region>
          <p className="text-muted-foreground truncate text-[0.62rem] font-medium tracking-[0.14em] uppercase">
            {t("brand.name")}
          </p>
          <p className="text-foreground/80 truncate text-[0.68rem] leading-none">
            {t("brand.tagline")}
          </p>
        </div>
      </div>

      <div className="flex shrink-0 items-stretch pr-1">
        <TitleBarButton
          label={t("chrome.minimize")}
          onClick={() => void withWindow((win) => win.minimize())}
        >
          <Minus className="size-3.5" strokeWidth={1.75} />
        </TitleBarButton>
        <TitleBarButton
          label={maximized ? t("chrome.restore") : t("chrome.maximize")}
          onClick={() => void withWindow((win) => win.toggleMaximize())}
        >
          {maximized ? (
            <span className="relative size-3" aria-hidden>
              <span className="border-current absolute inset-[1.5px] translate-x-[1.5px] translate-y-[-1.5px] border" />
              <span className="border-current bg-background absolute inset-[1.5px] border" />
            </span>
          ) : (
            <Square className="size-3" strokeWidth={1.75} />
          )}
        </TitleBarButton>
        <TitleBarButton
          label={t("chrome.close")}
          destructive
          onClick={() => void withWindow((win) => win.close())}
        >
          <X className="size-3.5" strokeWidth={1.75} />
        </TitleBarButton>
      </div>
    </header>
  );
}

function TitleBarButton({
  label,
  onClick,
  destructive,
  children,
}: {
  label: string;
  onClick: () => void;
  destructive?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={cn(
        "inline-flex w-11 items-center justify-center rounded-md text-muted-foreground",
        "hover:bg-muted/75 hover:text-foreground focus-visible:bg-muted/75 focus-visible:outline-none",
        destructive &&
          "hover:bg-destructive hover:text-destructive-foreground focus-visible:bg-destructive focus-visible:text-destructive-foreground",
      )}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
