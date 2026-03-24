import { Component, JSX } from 'solid-js';

export const Label: Component<JSX.LabelHTMLAttributes<HTMLLabelElement>> = (props) => {
  return (
    <label
      {...props}
      class={`block font-heading font-bold uppercase text-sm mb-2 ${props.class || ''}`}
    />
  );
};
