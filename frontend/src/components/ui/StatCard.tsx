import { Component, JSX } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from './Card';

interface StatCardProps {
  title: string;
  value: string | number;
  icon?: string;
  trend?: {
    value: number;
    label: string;
  };
  variant?: 'primary' | 'secondary' | 'accent' | 'default';
  children?: JSX.Element;
  class?: string;
}

export const StatCard: Component<StatCardProps> = (props) => {
  const variantClasses = {
    primary: 'bg-primary border-primary',
    secondary: 'bg-secondary text-white border-secondary',
    accent: 'bg-accent-yellow border-accent-yellow',
    default: 'bg-white border-black',
  };

  return (
    <Card 
      variant={props.variant === 'accent' ? 'primary' : props.variant} 
      class={`transform hover:-translate-y-2 hover:shadow-brutal-xl transition-all duration-200 ${props.variant === 'accent' ? 'bg-accent-yellow' : ''} ${props.class || ''}`}
    >
      <CardHeader>
        <div class="flex items-center justify-between">
          <CardTitle>{props.title}</CardTitle>
          {props.icon && <span class="text-4xl">{props.icon}</span>}
        </div>
      </CardHeader>
      <CardContent>
        <div class="text-display-1 font-heading font-black">
          {props.value}
        </div>
        
        {props.trend && (
          <div class="flex items-center gap-2 mt-3">
            <span class={`text-sm font-bold ${props.trend.value >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              {props.trend.value >= 0 ? '↗' : '↘'} {Math.abs(props.trend.value)}%
            </span>
            <span class="text-xs text-neutral-darkGray">
              {props.trend.label}
            </span>
          </div>
        )}
        
        {props.children}
      </CardContent>
    </Card>
  );
};

export default StatCard;
