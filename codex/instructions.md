# Global Coding Instructions

## Tech Stack

- **Framework**: Next.js (App Router)
- **Language**: TypeScript (strict mode)
- **Styling**: Tailwind CSS + `cn()` (clsx + tailwind-merge)
- **Testing**: Vitest + React Testing Library
- **Runtime**: React 19, Node.js 20+

## Research First

Before implementing anything you're unsure about, research it first. Check official docs, GitHub issues, changelogs. 10 minutes of research saves 1 hour of guessing. Never wing it.

Key references:
- Next.js: https://nextjs.org/docs
- React: https://react.dev/reference
- Tailwind CSS: https://tailwindcss.com/docs
- TypeScript: https://www.typescriptlang.org/docs

---

## Component Architecture

### 1. Server Components by Default

Every component is a React Server Component unless it needs interactivity. Only add `"use client"` when the component requires:
- Event handlers (`onClick`, `onChange`, etc.)
- Hooks (`useState`, `useEffect`, `useRef`, etc.)
- Browser-only APIs (`window`, `localStorage`, `IntersectionObserver`, etc.)

Push `"use client"` boundaries as far down the tree as possible. A page should never be a Client Component — extract the interactive part into a child.

### 2. One Component Per File

Each file exports a single component as the default export. Colocate small helper components in the same file only if they're tightly coupled and not reused elsewhere.

File naming: `kebab-case.tsx` for components, matching the component name in PascalCase.
```
components/
  user-profile.tsx    → export default function UserProfile()
  search-input.tsx    → export default function SearchInput()
```

### 3. Props Design

- Keep props minimal. If a component takes more than 5 props, consider splitting it or using composition.
- Define props as a `type`, not an `interface`, colocated above the component.
- Use `ReactNode` for slot-like props (`children`, `header`, `footer`).
- Prefer composition over configuration: pass components as children rather than configuring behavior through many props.

```tsx
type UserCardProps = {
  name: string;
  email: string;
  avatar?: string;
  actions?: ReactNode;
};
```

### 4. Compound Components

For complex UI with shared state (tabs, accordions, selects), use the compound component pattern:

```tsx
<Tabs defaultValue="general">
  <TabsList>
    <TabsTrigger value="general">General</TabsTrigger>
    <TabsTrigger value="security">Security</TabsTrigger>
  </TabsList>
  <TabsContent value="general">...</TabsContent>
  <TabsContent value="security">...</TabsContent>
</Tabs>
```

---

## State Management

### 5. Derive State, Don't Duplicate It

Never store state that can be computed from other state. Use derived values instead.

```tsx
// Bad — duplicates and desyncs
const [fullName, setFullName] = useState(first + last);

// Good — derived, always in sync
const fullName = `${first} ${last}`;
```

Keep the number of `useState` calls minimal. If state A can be computed from state B, only store B.

### 6. State Dependencies Must Form a DAG

State dependencies must form a Directed Acyclic Graph — no circular dependencies. Data should flow in one direction: parent → child, server → client. If two pieces of state depend on each other, extract a single source of truth.

### 7. Minimize useEffect and useRef

`useEffect` is a last resort. Most "effects" are actually derived state, event handlers, or should be handled by the framework (server components, route handlers, server actions).

Before reaching for `useEffect`, ask:
- Can this be derived state?
- Can this be an event handler?
- Can this run on the server?

`useRef` should only be used for DOM access or values that must persist across renders without triggering re-renders.

### 8. Use Suspense

Prefer React `Suspense` boundaries for async data and lazy-loaded components. Wrap route segments and data-fetching components in `<Suspense>` with appropriate fallbacks (skeleton screens, not spinners where possible).

---

## UX & States

### 9. Handle All UI States

Every component that deals with async data must handle all states gracefully:

- **Loading**: Skeleton screens via `<Suspense>` fallbacks. Never show a blank screen.
- **Error**: Error Boundary with clear message and recovery action. Never show a broken UI.
- **Empty**: Meaningful empty states with guidance. Never show a blank container.
- **Success**: Provide feedback for user actions (toast, animation, state change).

### 10. Error Handling Strategy

- Use `error.tsx` files in route segments for Error Boundaries.
- Server Actions return a result object, never throw for expected errors:
  ```tsx
  type ActionResult<T = void> =
    | { success: true; data: T }
    | { success: false; error: string };
  ```
- Unexpected errors (network failures, bugs) should bubble up to the nearest Error Boundary.
- Log errors server-side. Never expose internal error details to the client.

---

## Styling

### 11. Tailwind + cn() Conventions

Use Tailwind utility classes as the sole styling method. No CSS modules, no inline styles, no `styled-components`.

Use `cn()` for conditional and merged class names:

```tsx
import { cn } from '@/lib/utils';

function Button({ variant, className, ...props }: ButtonProps) {
  return (
    <button
      className={cn(
        'rounded-md px-4 py-2 font-medium transition-colors',
        variant === 'primary' && 'bg-blue-600 text-white hover:bg-blue-700',
        variant === 'ghost' && 'hover:bg-gray-100',
        className,
      )}
      {...props}
    />
  );
}
```

Rules:
- Always accept and merge `className` prop on reusable components.
- Group related utilities visually: layout → spacing → typography → colors → effects.
- Extract repeated class combinations into components, not into `@apply`.
- Never use `@apply` in CSS files — it defeats the purpose of utility-first.

### 12. Responsive & Motion

- Mobile-first: start with base styles, add `sm:`, `md:`, `lg:` for larger screens.
- Respect `prefers-reduced-motion` with `motion-safe:` and `motion-reduce:` variants.
- Use Tailwind's `transition-*` utilities for simple transitions.

---

## TypeScript

### 13. Strict and Explicit

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "exactOptionalPropertyTypes": true
  }
}
```

- All functions must have explicit return types.
- Use `type` over `interface` unless declaration merging is needed.
- Use `import type { ... }` for type-only imports.
- No `any`. Use `unknown` and narrow with type guards.
- No non-null assertions (`!`). Handle `null`/`undefined` explicitly.

### 14. Naming Conventions

| Kind | Convention | Example |
|------|-----------|---------|
| Type / Interface | PascalCase | `UserProfile`, `ApiResponse<T>` |
| Variable / Function | camelCase | `getUserById`, `isLoading` |
| Constant (module-level) | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT`, `API_BASE_URL` |
| Component | PascalCase | `SearchInput`, `UserCard` |
| File (component) | kebab-case | `search-input.tsx`, `user-card.tsx` |
| File (utility) | kebab-case | `format-date.ts`, `use-debounce.ts` |

---

## Event Handlers

### 15. Inline Short Handler Functions

- **Short/simple logic**: write it inline. `onClick={() => setOpen(true)}` is fine.
- **Complex logic (5+ lines, reused, or with dependencies)**: extract to a named function with `handle` prefix (`handleSubmit`, `handleDelete`).
- Don't prematurely extract one-liner handlers into separate functions.

---

## Accessibility

### 16. a11y is Mandatory

- Semantic HTML: `<button>` for actions, `<a>` for navigation, `<label>` for inputs. Never `<div onClick>`.
- All interactive elements must be keyboard-accessible (Tab, Enter, Escape).
- All images need `alt` text (or `alt=""` if decorative).
- Icon-only buttons need `aria-label`.
- Color must never be the sole indicator — add text or icons.
- Minimum contrast ratio: 4.5:1 for normal text, 3:1 for large text.
- Visible focus states on all interactive elements (`:focus-visible`).
- Form inputs must have associated `<label>` elements.

---

## React Imports

### 17. Import React Types Directly

```tsx
// Good
import { useState, type ChangeEvent, type ReactNode } from 'react';

// Bad
React.ChangeEvent, React.ReactNode
```

Only use the `React` namespace when absolutely necessary (e.g., `React.createElement` in non-JSX contexts).

---

## Testing

### 18. Testing Strategy

Use Vitest + React Testing Library. Test behavior, not implementation.

**What to test:**
- User interactions (clicks, typing, form submission)
- Conditional rendering based on props and state
- Error states and edge cases
- Accessibility (elements have correct roles, labels)

**What NOT to test:**
- Implementation details (internal state, private methods)
- Styling / CSS classes
- Third-party library internals
- 1:1 snapshot tests (too brittle)

**Conventions:**
- File naming: `component-name.test.tsx`, colocated next to the component.
- Use `screen` queries in priority: `getByRole` > `getByLabelText` > `getByText` > `getByTestId`.
- `data-testid` is a last resort for elements without accessible names.
- Each test describes a user-visible behavior: `it('shows error message when submission fails')`.

```tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SearchInput from './search-input';

describe('SearchInput', () => {
  it('calls onSearch when the user submits a query', async () => {
    const onSearch = vi.fn();
    render(<SearchInput onSearch={onSearch} />);

    await userEvent.type(screen.getByRole('searchbox'), 'hello');
    await userEvent.click(screen.getByRole('button', { name: /search/i }));

    expect(onSearch).toHaveBeenCalledWith('hello');
  });
});
```

---

## Project Structure

```
src/
  app/                    # Next.js App Router (routes, layouts, pages)
    (auth)/               # Route groups
    api/                  # Route handlers
    layout.tsx
    page.tsx
  components/
    ui/                   # Primitive UI components (button, input, dialog)
    [feature]/            # Feature-specific components
  lib/                    # Utilities, helpers, configs
    utils.ts              # cn() and shared utilities
  types/                  # Shared type definitions
  hooks/                  # Custom hooks (client-side only)
  actions/                # Server Actions
```
