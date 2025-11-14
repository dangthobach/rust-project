import { Component } from 'solid-js';
import { Card, CardContent, Badge } from '~/components/ui';

interface TaskCardProps {
  title: string;
  description: string;
  priority: 'high' | 'medium' | 'low';
  dueDate: string;
  status: 'pending' | 'in-progress' | 'completed';
}

export const TaskCard: Component<TaskCardProps> = (props) => {
  const priorityColors = {
    high: 'bg-accent text-white',
    medium: 'bg-secondary text-black',
    low: 'bg-neutral-concrete text-black',
  };

  const statusColors = {
    pending: 'bg-neutral-beige text-black',
    'in-progress': 'bg-primary text-black',
    completed: 'bg-green-500 text-white',
  };

  return (
    <Card hoverable class="animate-slide-in">
      <CardContent class="p-4">
        <div class="flex items-start justify-between gap-2 mb-2">
          <h4 class="font-heading font-bold text-lg flex-1">{props.title}</h4>
          <Badge class={priorityColors[props.priority]}>{props.priority}</Badge>
        </div>

        <p class="text-sm text-neutral-darkGray mb-3">{props.description}</p>

        <div class="flex items-center justify-between pt-3 border-t-3 border-black">
          <Badge class={statusColors[props.status]}>{props.status}</Badge>
          <span class="text-xs font-mono text-neutral-gray">Due: {props.dueDate}</span>
        </div>
      </CardContent>
    </Card>
  );
};
