import type { ComponentType, ReactNode } from "react";

import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon: ComponentType<{ className?: string }>;
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center rounded-xl border border-dashed border-border/80 bg-muted/15 px-6 py-10 text-center",
        className,
      )}
    >
      <div className="bg-primary/10 text-primary mb-3 flex size-10 items-center justify-center rounded-full">
        <Icon className="size-5" aria-hidden />
      </div>
      <h3 className="text-sm font-medium text-balance">{title}</h3>
      <p className="text-muted-foreground mt-1 max-w-sm text-xs leading-relaxed text-pretty">
        {description}
      </p>
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
