import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '~/theme/utils';

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

export const Input: Component<InputProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'error', 'children']);
  return (
    <input
      class={cn('input', local.error && 'input-error', local.class)}
      {...others}
    />
  );
};

export const Textarea: Component<JSX.TextareaHTMLAttributes<HTMLTextAreaElement>> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children']);
  return (
    <textarea
      class={cn('textarea', local.class)}
      {...others}
    >
      {local.children}
    </textarea>
  );
};
