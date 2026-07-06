import type * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2 py-0.5 text-[0.68rem] font-semibold uppercase tracking-wide",
  {
    variants: {
      variant: {
        default: "border-transparent bg-secondary text-secondary-foreground",
        good: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
        fair: "border-amber-500/25 bg-amber-500/10 text-amber-700 dark:text-amber-300",
        poor: "border-rose-500/25 bg-rose-500/10 text-rose-700 dark:text-rose-300",
        info: "border-sky-500/25 bg-sky-500/10 text-sky-700 dark:text-sky-300",
        caution: "border-amber-500/25 bg-amber-500/10 text-amber-700 dark:text-amber-300",
        untrusted: "border-rose-500/25 bg-rose-500/10 text-rose-700 dark:text-rose-300",
        recommended: "border-amber-500/25 bg-amber-500/10 text-amber-700 dark:text-amber-200",
        active: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
        muted: "border-border bg-muted text-muted-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

function Badge({
  className,
  variant,
  ...props
}: React.ComponentProps<"span"> & VariantProps<typeof badgeVariants>) {
  return <span data-slot="badge" className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };
