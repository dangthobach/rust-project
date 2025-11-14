import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '~/theme/utils';

interface SpinnerProps extends JSX.HTMLAttributes<HTMLDivElement> {
  size?: 'sm' | 'md' | 'lg';
}

export const Spinner: Component<SpinnerProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'size']);
  const size = () => local.size || 'md';

  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };

  return (
    <div
      class={cn('spinner', sizeClasses[size()], local.class)}
      {...others}
    />
  );
};
