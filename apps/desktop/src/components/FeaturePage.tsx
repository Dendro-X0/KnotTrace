import type { ComponentType, ReactNode } from "react";

import { Badge, type badgeVariants } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
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
}: FeaturePageProps) {
  return (
    <Card className="min-h-0 flex-1 border-border/70 bg-card/80 shadow-sm">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <Icon className="size-4" />
          </div>
          <CardTitle>{title}</CardTitle>
        </div>
        {badge ? <Badge variant={badge.variant}>{badge.label}</Badge> : loading ? <Skeleton className="h-5 w-20 rounded-full" /> : null}
      </CardHeader>
      <CardContent className="flex min-h-0 flex-1 flex-col gap-3">
        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-4 w-full max-w-xl" />
            <Skeleton className="h-4 w-2/3" />
          </div>
        ) : (
          <CardDescription>{description}</CardDescription>
        )}
        {error && <p className="text-destructive text-sm">{error}</p>}
        {children}
        {footer}
      </CardContent>
    </Card>
  );
}
