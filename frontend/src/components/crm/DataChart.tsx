import { Component, For } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';

interface ChartData {
  label: string;
  value: number;
  color?: string;
}

export const DataChart: Component = () => {
  const data: ChartData[] = [
    { label: 'Jan', value: 45, color: '#00FF00' },
    { label: 'Feb', value: 62, color: '#0080FF' },
    { label: 'Mar', value: 38, color: '#FFFF00' },
    { label: 'Apr', value: 78, color: '#FF10F0' },
    { label: 'May', value: 56, color: '#FF6B00' },
  ];

  const maxValue = Math.max(...data.map((d) => d.value));

  return (
    <Card>
      <CardHeader class="p-6 border-b-3 border-black">
        <CardTitle>Monthly Performance</CardTitle>
      </CardHeader>
      <CardContent class="p-6">
        <div class="flex items-end justify-between gap-2 h-64">
          <For each={data}>
            {(item) => {
              const heightPercent = (item.value / maxValue) * 100;

              return (
                <div class="flex-1 flex flex-col items-center gap-2">
                  <div class="relative w-full flex flex-col items-center justify-end flex-1">
                    <div
                      class="w-full border-5 border-black shadow-brutal transition-all hover:-translate-y-1 hover:shadow-brutal-lg cursor-pointer"
                      style={{
                        height: `${heightPercent}%`,
                        'background-color': item.color,
                      }}
                    >
                      <div class="absolute -top-8 left-1/2 -translate-x-1/2 font-heading font-bold text-lg">
                        {item.value}
                      </div>
                    </div>
                  </div>
                  <div class="font-heading text-xs font-bold uppercase text-center mt-2">
                    {item.label}
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </CardContent>
    </Card>
  );
};
