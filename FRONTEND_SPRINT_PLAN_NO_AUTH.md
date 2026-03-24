# Frontend Sprint Plan (No-Auth Scope, Keycloak PKCE Deferred)

Pham vi tai lieu:
- Khong tich hop API auth/user-auth o giai doan nay.
- Uu tien thong luong API + UI cho core business flows.
- Ke hoach de assign truc tiep cho team FE theo sprint/task.

## 1) Scope va non-scope

**In scope (lam ngay):**
- Dashboard, Clients, Tasks, Files, Reports, Notifications (no-auth mode).
- Shared component system de tai su dung cao.
- API integration layer + query/mutation hooks.
- Error/loading/empty/pending states day du.

**Out of scope (de sau):**
- `/api/auth/*`
- User auth lifecycle (login/register/logout/refresh)
- PKCE, token storage, RBAC UI gate theo token claims

## 2) Goi y role team (owner labels)

- `FE-Lead`: chot architecture, review PR, giai quyet dependency.
- `FE-Platform`: API client, query layer, shared components.
- `FE-Biz-1`: Tasks + Clients screens.
- `FE-Biz-2`: Files + Reports screens.
- `FE-QA`: test case, smoke E2E no-auth, regression checklist.

Neu team nho hon, 1 nguoi co the giu 2 owner labels.

## 3) Sprint breakdown (task-level, estimate, owner)

Quy uoc estimate:
- `S` = 0.5-1 day
- `M` = 1-2 days
- `L` = 2-3 days

### Sprint 0 - Foundation (2-3 ngay)

| ID | Task | Deliverable | Estimate | Owner goi y | Dependency |
|---|---|---|---|---|---|
| FE0-01 | Khoi tao no-auth app shell | app boot, route skeleton, env config | S | FE-Lead | none |
| FE0-02 | Tao `ApiClient` + error mapping chung | `api/client.ts`, retry/timeout, normalize error | M | FE-Platform | FE0-01 |
| FE0-03 | Setup query library + query key convention | query provider + key helpers | S | FE-Platform | FE0-02 |
| FE0-04 | Shared UI state kit | `LoadingState`, `EmptyState`, `ErrorState`, `ConfirmDialog` | M | FE-Platform | FE0-01 |
| FE0-05 | Definition of Done checklist template | PR checklist + screen checklist | S | FE-QA | FE0-01 |

### Sprint 1 - Files flow critical (3-4 ngay)

| ID | Task | Deliverable | Estimate | Owner goi y | Dependency |
|---|---|---|---|---|---|
| FE1-01 | Files list page | table/grid files + pagination + empty state | M | FE-Biz-2 | FE0-02, FE0-03 |
| FE1-02 | Reusable `FileUploader` | upload component co progress + validation | M | FE-Platform | FE0-04 |
| FE1-03 | Upload integration | call `/api/files/upload`, refresh list, queued status | M | FE-Biz-2 | FE1-01, FE1-02 |
| FE1-04 | Download URL flow | call `/api/files/:id/download-url`, open link, expiry UI | S | FE-Biz-2 | FE1-01 |
| FE1-05 | Thumbnail pending UI | badge/trang thai processing + polling nhe | M | FE-Biz-2 | FE1-03 |
| FE1-06 | Smoke test files flow | test checklist upload->thumbnail->download->delete | S | FE-QA | FE1-01..FE1-05 |

### Sprint 2 - Tasks + Clients core business (4-5 ngay)

| ID | Task | Deliverable | Estimate | Owner goi y | Dependency |
|---|---|---|---|---|---|
| FE2-01 | Reusable `DataTable` + `FilterBar` schema-driven | shared table/filter framework | L | FE-Platform | FE0-04 |
| FE2-02 | Tasks list + filter | `/api/tasks` list/filter/sort | M | FE-Biz-1 | FE2-01 |
| FE2-03 | Tasks create/update/complete/delete | modal/form + mutations + pending state | L | FE-Biz-1 | FE2-02 |
| FE2-04 | Clients list/search | `/api/clients`, `/api/clients/search` | M | FE-Biz-1 | FE2-01 |
| FE2-05 | Clients create/update/delete | form schema + mutation hooks | M | FE-Biz-1 | FE2-04 |
| FE2-06 | Cross-screen reusable forms | `EntityFormModal` cho tasks/clients | M | FE-Platform | FE2-03, FE2-05 |
| FE2-07 | QA regression sprint 2 | smoke scripts cho tasks/clients | S | FE-QA | FE2-02..FE2-06 |

### Sprint 3 - Reports + Dashboard + Notifications (3-4 ngay)

| ID | Task | Deliverable | Estimate | Owner goi y | Dependency |
|---|---|---|---|---|---|
| FE3-01 | Dashboard cards + activity feed | `/api/dashboard/stats`, `/api/dashboard/activity-feed` | M | FE-Biz-1 | FE0-03 |
| FE3-02 | Reusable `StatCard` + `AsyncJobToast` | shared stat/async feedback components | M | FE-Platform | FE0-04 |
| FE3-03 | Reports panel | trigger `/api/export/*`, show queued/success/fail | M | FE-Biz-2 | FE0-02, FE3-02 |
| FE3-04 | Recent download links widget | link list + expiry countdown | S | FE-Biz-2 | FE3-03 |
| FE3-05 | Notifications inbox | list/mark-read/delete | M | FE-Biz-1 | FE0-03 |
| FE3-06 | End-to-end smoke suite no-auth | e2e flow set cho release | M | FE-QA | FE3-01..FE3-05 |

### Sprint 4 - Hardening + handover cho PKCE phase (2-3 ngay)

| ID | Task | Deliverable | Estimate | Owner goi y | Dependency |
|---|---|---|---|---|---|
| FE4-01 | Perf tune list screens | memoization, virtualization where needed | M | FE-Lead + FE-Platform | FE2-01, FE1-01 |
| FE4-02 | Error observability | global error logger + trace id surfacing | S | FE-Platform | FE0-02 |
| FE4-03 | API contract freeze no-auth | mapping endpoint/payload final | S | FE-Lead | FE1..FE3 |
| FE4-04 | PKCE-ready adapter seams | auth provider stub, interceptor hook points | S | FE-Platform | FE0-02 |
| FE4-05 | Release checklist + handover docs | runbook + known gaps | S | FE-QA + FE-Lead | FE4-01..FE4-04 |

## 4) Assignment template (copy de giao task)

```
Task ID:
Title:
Owner:
Estimate:
Dependencies:
API endpoints:
Acceptance criteria:
- [ ] Loading state
- [ ] Empty state
- [ ] Error state
- [ ] Pending state for mutation
- [ ] Reuse shared component (khong viet moi neu da co)
QA notes:
```

## 5) Uu tien component reuse (bat buoc)

Tat ca task UI phai uu tien dung lai component da co theo thu tu:
1. `DataTable`, `FilterBar`, `SearchInput`
2. `EntityFormModal`, `ConfirmDialog`
3. `StatusBadge`, `AsyncJobToast`, `StatCard`
4. `LoadingState`, `EmptyState`, `ErrorState`

Neu can component moi, owner phai:
- Mo ta ly do tai sao component hien tai khong du.
- Thiet ke component theo generic props (khong domain-coupled).
- Add docs nho trong story/readme cua component.

## 6) Risk va giam thieu nghen

- Risk: backend protected endpoints tra 401 trong no-auth mode.  
  Mitigation: `ApiClient` co mock fallback de unblock UI implementation.

- Risk: duplicate component do chia team song song.  
  Mitigation: FE-Platform review gate bat buoc cho moi UI primitive PR.

- Risk: regressions khi mutate nhieu man hinh.  
  Mitigation: query key convention + shared mutation invalidation helpers.
