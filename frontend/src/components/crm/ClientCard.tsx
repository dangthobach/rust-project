import { Component } from 'solid-js';
import { Card, CardContent, Badge, Button } from '~/components/ui';

interface ClientCardProps {
  name: string;
  email: string;
  phone: string;
  status: 'active' | 'inactive';
  lastContact: string;
}

export const ClientCard: Component<ClientCardProps> = (props) => {
  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase();
  };

  return (
    <Card hoverable class="animate-slide-in">
      <CardContent class="p-6">
        <div class="flex items-start gap-4 mb-4">
          <div class="h-16 w-16 flex-shrink-0 border-3 border-black bg-primary shadow-brutal-sm flex items-center justify-center">
            <span class="font-heading text-xl font-black">{getInitials(props.name)}</span>
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="font-heading text-xl font-bold uppercase mb-1 truncate">{props.name}</h3>
            <p class="text-sm text-neutral-darkGray truncate">{props.email}</p>
          </div>
          <Badge variant={props.status === 'active' ? 'success' : 'default'}>
            {props.status}
          </Badge>
        </div>

        <div class="space-y-2 border-t-3 border-black pt-4">
          <div class="flex items-center gap-2">
            <span>📧</span>
            <span class="text-sm truncate">{props.email}</span>
          </div>
          <div class="flex items-center gap-2">
            <span>📞</span>
            <span class="text-sm">{props.phone}</span>
          </div>
          <div class="flex items-center gap-2">
            <span>🕒</span>
            <span class="text-sm text-neutral-darkGray">Last contact: {props.lastContact}</span>
          </div>
        </div>

        <div class="flex gap-2 mt-4">
          <Button size="sm" variant="secondary" fullWidth>
            View
          </Button>
          <Button size="sm" variant="ghost" fullWidth>
            Edit
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};
