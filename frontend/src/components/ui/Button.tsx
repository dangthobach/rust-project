import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '~/theme/utils';

type ButtonVariant = 'primary' | 'secondary' | 'accent' | 'ghost';
type ButtonVariantExtended = ButtonVariant | 'danger' | 'destructive' | 'outline';
type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariantExtended;
  size?: ButtonSize;
  fullWidth?: boolean;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'size', 'fullWidth', 'class', 'children']);
  const variant = () => local.variant || 'primary';
  const size = () => local.size || 'md';

  const variantClasses = {
    primary: 'btn-primary',
    secondary: 'btn-secondary',
    accent: 'btn-accent',
    ghost: 'btn bg-transparent shadow-none hover:bg-neutral-beige',
    danger: 'btn-secondary bg-red-500 text-white hover:bg-red-600',
    destructive: 'btn-secondary bg-red-500 text-white hover:bg-red-600',
    outline: 'btn bg-white shadow-none hover:bg-neutral-beige',
  };

  const sizeClasses = {
    xs: 'btn-sm text-xs px-2 py-1',
    sm: 'btn-sm',
    md: '',
    lg: 'btn-lg',
  };

  return (
    <button
      class={cn(
        'btn',
        variantClasses[variant()],
        sizeClasses[size()],
        local.fullWidth && 'w-full',
        local.class,
      )}
      {...others}
    >
      {local.children}
    </button>
  );
};
