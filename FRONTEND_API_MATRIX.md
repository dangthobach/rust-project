# Frontend API Matrix (Flow-First, No-Auth FE)

Muc tieu: thong luong API <-> UI truoc, giam nghen khi FE chia viec.  
Pham vi: map endpoint, payload, trang thai UI, va bo component tai su dung.

## 1) Nguyen tac thuc thi no-auth cho FE

- FE chay `NO_AUTH=true` (khong hien login UI), nhung API layer van giu cho san header auth de bat lai sau.
- Backend hien tai phan lon endpoint la protected; de thong luong nhanh:
  - Uu tien test luong public + luong noi bo qua mock/stub response trong FE.
  - Với luong protected, dung 1 adapter trong FE (`ApiClient`) de mock fallback khi 401.
- Khong hard-code auth vao component; auth chi ton tai trong `api/interceptors`.

## 2) API Matrix theo man hinh

| Screen | Endpoint | Method | Request Payload/Params | Response chinh | UI States can co |
|---|---|---|---|---|---|
| Health/Bootstrap | `/health` | GET | none | `{ status, timestamp, ... }` | loading app, healthy badge, retry when fail |
| Dashboard | `/api/dashboard/stats` | GET | none | stats cards | skeleton cards, empty-zero state, error banner |
| Dashboard | `/api/dashboard/activity-feed` | GET | `page, limit` | activity list | list loading, empty feed, pagination loading |
| Users (self) | `/api/users/me` | GET | none | user profile | profile skeleton, 404 fallback |
| Users (self) | `/api/users/:id` | PATCH | `{ full_name?, avatar_url? }` | updated user | saving, dirty form, success toast, inline validation |
| Users (self) | `/api/users/password` | POST | `{ current_password, new_password }` | `{ message }` | submit loading, password strength, error hint |
| Admin Users | `/api/admin/users` | GET | `search, role, limit, offset` | `User[]` | table skeleton, no-data, filter loading |
| Admin Users | `/api/admin/users` | POST | `{ email, full_name, password, role? }` | `User` | create modal loading, validation, optimistic insert optional |
| Admin Users | `/api/admin/users/:id` | PATCH | `{ email?, full_name?, role?, avatar_url? }` | `User` | row saving state, partial update feedback |
| Admin Users | `/api/admin/users/:id` | DELETE | none | `204` | confirm dialog, pending delete, undo toast (FE-side) |
| Clients | `/api/clients` | GET | `status, limit, offset` | `Client[]` | table loading, empty-state CTA |
| Clients | `/api/clients/search` | GET | `search_term, limit` | `Client[]` | debounced loading, no match, clear filter |
| Clients | `/api/clients` | POST | `CreateClientCommand` | `Client` | modal loading, field errors, success refresh |
| Clients | `/api/clients/:id` | PATCH | `UpdateClientCommand` | `Client` | detail saving, stale badge when refetching |
| Clients | `/api/clients/:id` | DELETE | none | `204` | confirm + destructive loading |
| Tasks | `/api/tasks` | GET | `status, priority, assigned_to, client_id, limit, offset` | `Task[]` | board/list skeleton, filtered empty state |
| Tasks | `/api/tasks` | POST | `{ title, description?, status?, priority?, assigned_to?, client_id?, due_date? }` | `Task` | create drawer loading, optimistic add option |
| Tasks | `/api/tasks/:id` | PATCH | same shape as create (optional fields) | `Task` | row/card saving spinner |
| Tasks | `/api/tasks/:id/complete` | POST | none | `Task` | instant check animation, revert on fail |
| Tasks | `/api/tasks/:id` | DELETE | none | `204` | confirm + pending UI |
| Files (legacy) | `/api/files` | GET | `page, limit` | paginated files | gallery/table skeleton, empty upload state |
| Files (legacy) | `/api/files/upload` | POST multipart | form-data field `file` | `File` | upload progress, size/type validation, queued badge |
| Files (legacy) | `/api/files/:id/download-url` | GET | none | `{ download_url, expires_in }` | "generating link", expiry countdown |
| Files (legacy) | `/api/files/:id/download` | GET | none | stream or `{download_url}` | open file or redirect, fallback message |
| Files (legacy) | `/api/files/:id` | DELETE | none | `{ message }` | delete pending + remove from list |
| File System (CQRS) | `/api/fs/files` | GET | paging/filter | file list dto | file explorer loading, virtualized list |
| File System (CQRS) | `/api/fs/files` | POST | create file payload | created file | create pending + auto-focus new row |
| File System (CQRS) | `/api/fs/files/:id/move` | PATCH | destination folder id | updated file | drag-drop pending state |
| File System (CQRS) | `/api/fs/files/:id/rename` | PATCH | `{ name }` | updated file | inline edit saving state |
| File System (CQRS) | `/api/fs/folders` | POST | create folder payload | created folder | tree optimistic node insert |
| Reports | `/api/export/clients` | GET | `format=csv/json,status?,start_date?,end_date?` | direct file + async job queued | export pending, queued toast, download panel |
| Reports | `/api/export/tasks` | GET | same as clients | direct file + async job queued | same states |
| Reports | `/api/export/users` | GET | `format=csv/json,status?` | direct file + async job queued | admin gate + export states |
| Reports | `/api/export/dashboard-report` | GET | none | json file + async job queued | quick download + queued notice |
| Notifications | `/api/notifications` | GET | page/filter | notification list | bell badge loading, empty inbox |
| Notifications | `/api/notifications/mark-read` | POST | ids or all flag | success | optimistic mark-read |
| Notifications | `/api/notifications/:id` | DELETE | none | success | dismiss animation + undo |

## 3) Reusable Component Matrix (bat buoc de tranh nghen)

| Component | Dung cho screen | Props/Contract can co | Ghi chu tai su dung |
|---|---|---|---|
| `PageLayout` | tat ca | `title, actions, filters, children` | 1 khung chung cho heading + toolbar |
| `DataTable` | users/clients/tasks/files | `columns, rows, loading, emptyState, rowActions, pagination` | khong embed domain logic vao table |
| `FilterBar` | list pages | `schema, values, onChange, onReset` | dung schema-driven de tai su dung |
| `SearchInput` | clients/tasks/files | `value, onChange, debounceMs` | shared debounce behavior |
| `EntityFormModal` | create/update users/clients/tasks | `fieldsSchema, initialValues, onSubmit, submitting, errors` | render form tu schema, giam duplicate |
| `ConfirmDialog` | delete/complete actions | `title, description, confirmLabel, loading` | 1 component cho tat ca destructive actions |
| `StatusBadge` | cards/table rows | `status, variantMap` | map status enum trung tam |
| `AsyncJobToast` | upload/export | `jobType, state, detail` | thong nhat feedback queue background |
| `FileUploader` | files page | `accept, maxSize, onUpload, progress` | reuse cho avatar/file business |
| `DownloadLinkCell` | files/reports | `url, expiresIn, onRefresh` | xu ly countdown/refresh link |
| `StatCard` | dashboard | `label, value, delta, loading` | dung cho stats va analytics |
| `ErrorState` + `EmptyState` | tat ca | `title, message, action` | bo UI state tong quat |

## 4) Data Layer Reuse (khuyen nghi bat buoc)

- Folder:
  - `src/api/<domain>.api.ts` (HTTP adapter)
  - `src/queries/<domain>.queries.ts` (query keys + hooks)
  - `src/mutations/<domain>.mutations.ts`
  - `src/types/<domain>.ts`
- Query key convention:
  - `['users', params]`, `['clients', params]`, `['tasks', params]`, `['files', params]`
  - detail key: `['users', 'detail', id]` ...
- Mutation policy:
  - Create/Delete: invalidate list key
  - Update: patch optimistic detail + invalidate list nhe
  - Upload/Export: dung `job state` store rieng de UI track background process

## 5) Priority backlog de thong luong nhanh

1. `Files + Upload + Download-url + thumbnail state` (critical integration flow).  
2. `Tasks list/create/update/complete` (core business flow).  
3. `Clients list/search/create/update/delete`.  
4. `Dashboard cards + activity feed`.  
5. `Reports export panel` (show queued + latest links).  
6. `Admin users + RBAC pages` (sau khi luong chinh on dinh).

## 6) Definition of Done cho moi screen FE

- Co du 4 state: `loading`, `success`, `empty`, `error`.
- Tat ca action co pending feedback (button spinner/row shimmer).
- Error message hien thi theo API error mapping chung.
- Khong call API truc tiep trong component presentational (chi qua hooks/service layer).
- Khong duplicate UI control (table/form/dialog phai dung component shared).
