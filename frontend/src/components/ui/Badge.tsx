import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '~/theme/utils';

type BadgeVariant =
  | 'primary'
  | 'secondary'
  | 'success'
  | 'warning'
  | 'info'
  | 'error'
  | 'neutral'
  | 'default'
  | 'danger'
  | 'destructive';

interface BadgeProps extends JSX.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

export const Badge: Component<BadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'class', 'children']);
  const variant = () => local.variant || 'neutral';

  const variantClasses = {
    primary: 'badge-primary',
    secondary: 'badge-secondary',
    success: 'badge-success',
    warning: 'badge-warning',
    info: 'badge-primary',
    error: 'badge-error',
    neutral: 'badge bg-neutral-concrete text-black',
    default: 'badge bg-neutral-concrete text-black',
    danger: 'badge-error',
    destructive: 'badge-error',
  };

  return (
    <span class={cn('badge', variantClasses[variant()], local.class)} {...others}>
      {local.children}
    </span>
  );
};
