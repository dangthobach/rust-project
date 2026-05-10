import { Component, createSignal, createEffect } from 'solid-js';
import { Button, Input, Label, Card, CardContent, CardHeader, CardTitle } from '~/components/ui';
import type { RoleDto, CreateRoleInput } from '~/lib/api';

interface RoleFormProps {
  initial?: Partial<RoleDto>;
  loading?: boolean;
  submitLabel?: string;
  onSubmit: (data: CreateRoleInput) => void;
  onCancel?: () => void;
}

export const RoleForm: Component<RoleFormProps> = (props) => {
  const [slug, setSlug] = createSignal(props.initial?.slug ?? '');
  const [description, setDescription] = createSignal(props.initial?.description ?? '');
  const [isActive, setIsActive] = createSignal(props.initial?.is_active ?? true);
  const [errors, setErrors] = createSignal<Record<string, string>>({});

  createEffect(() => {
    if (props.initial) {
      setSlug(props.initial.slug ?? '');
      setDescription(props.initial.description ?? '');
      setIsActive(props.initial.is_active ?? true);
    }
  });

  function validate(): boolean {
    const errs: Record<string, string> = {};
    if (!slug().trim()) errs.slug = 'Slug không được để trống';
    else if (!/^[a-z0-9_:-]+$/.test(slug()))
      errs.slug = 'Chỉ dùng chữ thường, số, dấu gạch dưới, gạch ngang hoặc dấu hai chấm';
    setErrors(errs);
    return Object.keys(errs).length === 0;
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!validate()) return;
    props.onSubmit({ slug: slug().trim(), description: description().trim() || undefined, is_active: isActive() });
  }

  return (
    <Card class="border-[3px] border-black shadow-brutal max-w-lg">
      <CardHeader>
        <CardTitle class="font-mono text-xs uppercase tracking-widest">
          Thông tin Role
        </CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} class="flex flex-col gap-4">
          <div>
            <Label for="slug" class="font-heading text-[10px] font-black uppercase">
              Slug <span class="text-red-600">*</span>
            </Label>
            <Input
              id="slug"
              type="text"
              placeholder="vd: admin, manager, viewer"
              value={slug()}
              onInput={(e: any) => setSlug(e.currentTarget.value)}
              class={errors().slug ? 'border-red-500' : ''}
            />
            {errors().slug && (
              <p class="mt-1 font-mono text-[10px] text-red-600">{errors().slug}</p>
            )}
          </div>

          <div>
            <Label for="description" class="font-heading text-[10px] font-black uppercase">
              Mô tả
            </Label>
            <Input
              id="description"
              type="text"
              placeholder="Mô tả ngắn về role này..."
              value={description()}
              onInput={(e: any) => setDescription(e.currentTarget.value)}
            />
          </div>

          <div class="flex items-center gap-2">
            <input
              id="is_active"
              type="checkbox"
              checked={isActive()}
              onChange={(e: any) => setIsActive(e.currentTarget.checked)}
              class="h-4 w-4 cursor-pointer accent-black"
            />
            <Label for="is_active" class="cursor-pointer font-heading text-[10px] font-black uppercase">
              Kích hoạt
            </Label>
          </div>

          <div class="flex justify-end gap-2 pt-2">
            {props.onCancel && (
              <Button type="button" variant="secondary" size="sm" onClick={props.onCancel}>
                Huỷ
              </Button>
            )}
            <Button type="submit" variant="primary" size="sm" disabled={props.loading}>
              {props.loading ? '...' : (props.submitLabel ?? 'Lưu')}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
};
