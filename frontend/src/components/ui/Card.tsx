import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '~/theme/utils';

interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'primary' | 'secondary';
  hoverable?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg';
}

export const Card: Component<CardProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'hoverable', 'padding', 'class', 'children']);
  const variant = () => local.variant || 'default';
  const padding = () => local.padding || 'md';

  const variantClasses = {
    default: 'card',
    primary: 'card-primary',
    secondary: 'card-secondary',
  };

  const paddingClasses = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  };

  return (
    <div
      class={cn(
        variantClasses[variant()],
        local.hoverable && 'card-hover cursor-pointer',
        paddingClasses[padding()],
        local.class,
      )}
      {...others}
    >
      {local.children}
    </div>
  );
};

interface SimpleProps extends JSX.HTMLAttributes<HTMLDivElement> {}

export const CardHeader: Component<SimpleProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);
  return (
    <div class={cn('mb-4', local.class)} {...others}>
      {local.children}
    </div>
  );
};

export const CardTitle: Component<JSX.HTMLAttributes<HTMLHeadingElement>> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);
  return (
    <h3 class={cn('font-heading text-heading-3 font-bold uppercase', local.class)} {...others}>
      {local.children}
    </h3>
  );
};

export const CardContent: Component<SimpleProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);
  return (
    <div class={cn('', local.class)} {...others}>
      {local.children}
    </div>
  );
};

export const CardFooter: Component<SimpleProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);
  return (
    <div class={cn('mt-4 flex items-center gap-2', local.class)} {...others}>
      {local.children}
    </div>
  );
};
