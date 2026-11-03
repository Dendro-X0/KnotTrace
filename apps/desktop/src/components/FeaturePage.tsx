import type { ComponentType, ReactNode } from "react";

import { Badge, type badgeVariants } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type { VariantProps } from "class-variance-authority";

interface FeaturePageProps {
  title: string;
  description: string;
  icon: ComponentType<{ className?: string }>;
  badge?: { label: string; variant: VariantProps<typeof badgeVariants>["variant"] };
  error?: string | null;
  loading?: boolean;
  children: ReactNode;
  footer?: ReactNode;
  /**
   * `page` — standalone card title (rare).
   * `embedded` — AppHeader already owns the page title; show badge + description only.
   */
  chrome?: "page" | "embedded";
}

export function FeaturePage({
  title,
  description,
  icon: Icon,
  badge,
  error,
  loading,
  children,
  footer,
  chrome = "embedded",
}: FeaturePageProps) {
  const embedded = chrome === "embedded";

  return (
    <Card className="min-h-0 border-border/60 bg-card/70 shadow-sm">
      <CardHeader className={cn(embedded && "space-y-0 pb-2")}>
        {embedded ? (
          <div className="flex flex-wrap items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-7 items-center justify-center rounded-lg">
              <Icon className="size-3.5" aria-hidden />
            </div>
            <span className="sr-only">{title}</span>
            {badge ? (
              <Badge variant={badge.variant}>{badge.label}</Badge>
            ) : loading ? (
              <Skeleton className="h-5 w-20 rounded-full" />
            ) : null}
          </div>
        ) : (
          <>
            <div className="flex items-center gap-2">
              <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
                <Icon className="size-4" />
              </div>
              <CardTitle>{title}</CardTitle>
            </div>
            {badge ? (
              <Badge variant={badge.variant}>{badge.label}</Badge>
            ) : loading ? (
              <Skeleton className="h-5 w-20 rounded-full" />
            ) : null}
          </>
        )}
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-4 w-full max-w-xl" />
            <Skeleton className="h-4 w-2/3" />
          </div>
        ) : (
          <CardDescription className={cn(embedded && "text-sm leading-relaxed")}>
            {description}
          </CardDescription>
        )}
        {error && <p className="text-destructive text-sm">{error}</p>}
        {children}
        {footer}
      </CardContent>
    </Card>
  );
}
