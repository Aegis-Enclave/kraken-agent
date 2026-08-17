---
name: shadcn-ui
description: Use shadcn/ui component library for building modern, accessible React applications with Next.js or Vite. Covers installation, all major components (forms, dialogs, tables, charts, navigation, overlays), theming with CSS variables, and integration patterns with Tailwind CSS. NOT for Vue or non-Tailwind projects — use radix-ui primitives directly in those cases.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [react, nextjs, vite, tailwind, component-library, ui, accessible, typescript, shadcn]
    category: fullstack-webdev
---

# shadcn/ui

shadcn/ui (https://ui.shadcn.com) is a collection of re-usable components built with Radix UI and Tailwind CSS. Unlike a traditional component library, you copy-paste components into your project — giving you full ownership and easy customization. Components are accessible by default (Radix primitives), themeable via CSS variables, and work with Next.js (App Router) or Vite.

## When to use this skill

Load this skill when a user wants to:

- Build a modern React UI with shadcn/ui components
- Integrate shadcn/ui with Next.js (App Router or Pages Router) or Vite
- Customize the default theme, colors, and CSS variables
- Use advanced components like Data Tables, Charts, Calendars, or Tiptap editor
- Build accessible forms with React Hook Form + Zod validation
- Create overlays (dialogs, drawers, sheets, tooltips) with proper focus management

## When NOT to use this skill

- User wants Vue/Svelte components — shadcn/ui is React-only; use Headless UI or Radix primitives directly
- User is not using Tailwind CSS — shadcn/ui requires Tailwind; consider Radix UI primitives + custom CSS
- User wants a server-side rendered component library for non-React frameworks

## Prerequisites

- Node.js 18+ with npm/pnpm/yarn/bun
- A React project with Tailwind CSS configured:
  ```bash
  # Next.js
  npx create-next-app@latest my-app --typescript --tailwind --eslint
  
  # Vite
  npm create vite@latest my-app -- --template react-ts
  npx tailwindcss init -p
  ```
- Understanding of React hooks and TypeScript basics

## Quick Start

### 1. Initialize shadcn/ui in an existing project

```bash
# In your project directory
npx shadcn@latest init

# Choose defaults or customize:
# - Style: Default
# - Base color: Neutral or Stone
# - CSS variables: Yes
# - CSS variables prefix: (leave default)
# - Customize default colors: No
# - Use CSS variables for colors: Yes
```

### 2. Add components

```bash
# Add individual components
npx shadcn@latest add button
npx shadcn@latest add card
npx shadcn@latest add dialog

# Add multiple at once
npx shadcn@latest add table avatar dropdown-menu tabs

# Add all components
npx shadcn@latest add -a
```

### 3. Use in your app

```tsx
import { Button } from "@/components/ui/button"
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "@/components/ui/card"

export default function Page() {
  return (
    <div className="container mx-auto py-10">
      <Card className="max-w-md">
        <CardHeader>
          <CardTitle>Welcome</CardTitle>
          <CardDescription>Get started with shadcn/ui</CardDescription>
        </CardHeader>
        <CardContent>
          <p>Build accessible UIs with ease.</p>
        </CardContent>
        <CardFooter>
          <Button>Get Started</Button>
        </CardFooter>
      </Card>
    </div>
  )
}
```

## Component Categories

### Forms & Input

| Component | Command | Description |
|-----------|---------|-------------|
| Button | `npx shadcn@latest add button` | Primary interaction element, multiple variants |
| Input | `npx shadcn@latest add input` | Text input field |
| Label | `npx shadcn@latest add label` | Accessible form labels |
| Textarea | `npx shadcn@latest add textarea` | Multi-line text input |
| Checkbox | `npx shadcn@latest add checkbox` | Boolean selection |
| Radio Group | `npx shadcn@latest add radio-group` | Single selection from options |
| Select | `npx shadcn@latest add select` | Dropdown selection |
| Switch | `npx shadcn@latest add switch` | Toggle on/off |
| Slider | `npx shadcn@latest add slider` | Range selection |
| Input OTP | `npx shadcn@latest add input-otp` | One-time password inputs |
| Form | `npx shadcn@latest add form` | Complete form with React Hook Form + Zod |
| Date Picker | `npx shadcn@latest add calendar` | Date selection with calendar popover |
| Combobox | `npx shadcn@latest add command` | Searchable dropdown (Command component) |

**Form pattern with Zod validation:**

```tsx
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form"

const formSchema = z.object({
  username: z.string().min(2, "Username must be at least 2 characters"),
  email: z.string().email("Invalid email address"),
})

export function ProfileForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: { username: "", email: "" },
  })

  function onSubmit(values: z.infer<typeof formSchema>) {
    console.log(values)
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl><Input {...field} /></FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit">Submit</Button>
      </form>
    </Form>
  )
}
```

### Navigation & Layout

| Component | Command | Description |
|-----------|---------|-------------|
| Navigation Menu | `npx shadcn@latest add navigation-menu` | Full navigation bar with dropdowns |
| Tabs | `npx shadcn@latest add tabs` | Panel switching tabs |
| Breadcrumb | `npx shadcn@latest add breadcrumb` | Hierarchical navigation path |
| Sidebar | `npx shadcn@latest add sidebar` | Collapsible sidebar navigation |
| Sheet | `npx shadcn@latest add sheet` | Slide-out panel (mobile menu) |
| Separator | `npx shadcn@latest add separator` | Horizontal/vertical divider |
| Aspect Ratio | `npx shadcn@latest add aspect-ratio` | Enforce aspect ratios |

**Navigation Menu example:**

```tsx
import { NavigationMenu, NavigationMenuList, NavigationMenuItem, NavigationMenuLink } from "@/components/ui/navigation-menu"

export function Nav() {
  return (
    <NavigationMenu>
      <NavigationMenuList>
        <NavigationMenuItem>
          <NavigationMenuLink href="/">Home</NavigationMenuLink>
        </NavigationMenuItem>
        <NavigationMenuItem>
          <NavigationMenuLink href="/about">About</NavigationMenuLink>
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  )
}
```

### Data Display

| Component | Command | Description |
|-----------|---------|-------------|
| Table | `npx shadcn@latest add table` | Data table with sorting/selection |
| Data Table | `npx shadcn@latest add data-table` | Full-featured table with pagination, filtering |
| Card | `npx shadcn@latest add card` | Container for grouped content |
| Avatar | `npx shadcn@latest add avatar` | User image with fallback |
| Badge | `npx shadcn@latest add badge` | Label/tag indicator |
| Chip | `npx shadcn@latest add chip` | Selectable tag (used in command) |
| Scroll Area | `npx shadcn@latest add scroll-area` | Custom scrollbar styling |
| Timeline | `npx shadcn@latest add timeline` | Chronological event list |
| Calendar | `npx shadcn@latest add calendar` | Date grid display |

**Data Table with TanStack Table:**

```tsx
"use client"

import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table"

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
}

export function DataTable<TData, TValue>({ columns, data }: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id}>
                  {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  )
}
```

### Overlays & Dialogs

| Component | Command | Description |
|-----------|---------|-------------|
| Dialog | `npx shadcn@latest add dialog` | Modal dialog with backdrop |
| Drawer | `npx shadcn@latest add drawer` | Slide-in panel (mobile-first alternative to sheet) |
| Alert Dialog | `npx shadcn@latest add alert-dialog` | Destructive confirmation dialogs |
| Sheet | `npx shadcn@latest add sheet` | Slide-out drawer (same as drawer) |
| Popover | `npx shadcn@latest add popover` | Floating content on click |
| Tooltip | `npx shadcn@latest add tooltip` | Hover information tooltip |
| Context Menu | `npx shadcn@latest add context-menu` | Right-click menu |
| Command | `npx shadcn@latest add command` | Command palette / combobox |

**Dialog pattern:**

```tsx
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"

export function DeleteConfirmation({ onConfirm }: { onConfirm: () => void }) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="destructive">Delete</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Are you sure?</DialogTitle>
          <DialogDescription>This action cannot be undone.</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline">Cancel</Button>
          <Button variant="destructive" onClick={onConfirm}>Delete</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
```

### Feedback & Status

| Component | Command | Description |
|-----------|---------|-------------|
| Alert | `npx shadcn@latest add alert` | Informational message with variants |
| Alert Dialog | `npx shadcn@latest add alert-dialog` | Blocking alert with confirmation |
| Toast | `npx shadcn@latest add toast` | Temporary notification |
| Skeleton | `npx shadcn@latest add skeleton` | Loading placeholder |
| Progress | `npx shadcn@latest add progress` | Progress bar |
| Spinner | `npx shadcn@latest add spinner` | Loading indicator |
| Empty State | `npx shadcn@latest add empty-state` | No data placeholder |
| Result | `npx shadcn@latest add result` | Success/error result display |

**Toast pattern (with Sonner):**

```tsx
import { Toaster } from "@/components/ui/sonner" // or @/components/ui/toaster
import { toast } from "sonner"

toast.success("Profile updated!")
toast.error("Failed to save changes")
toast("Event created", {
  description: "Friday, February 10 at 5:57 PM",
  action: { label: "Undo", onClick: () => {} },
})
```

### Charts & Visualization

| Component | Command | Description |
|-----------|---------|-------------|
| Chart | `npx shadcn@latest add chart` | Line, bar, pie charts using Recharts |

**Chart example:**

```tsx
import { ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import { Bar, BarChart, XAxis, YAxis } from "recharts"

const chartData = [
  { month: "January", sales: 186 },
  { month: "February", sales: 305 },
  { month: "March", sales: 237 },
]

const chartConfig = {
  sales: { label: "Sales", color: "hsl(var(--chart-1))" },
} satisfies ChartConfig

export function SalesChart() {
  return (
    <ChartContainer config={chartConfig} className="h-[200px] w-full">
      <BarChart data={chartData}>
        <XAxis dataKey="month" tickLine={false} tickMargin={10} />
        <YAxis tickLine={false} tickMargin={10} />
        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />
        <Bar dataKey="sales" fill="var(--color-sales)" radius={8} />
      </BarChart>
    </ChartContainer>
  )
}
```

### Specialized Components

| Component | Command | Description |
|-----------|---------|-------------|
| Calendar | `npx shadcn@latest add calendar` | Date picker calendar |
| Date Picker | `npx shadcn@latest add date-picker` | Calendar + input combination |
| Carousel | `npx shadcn@latest add carousel` | Image/content slider |
| Accordion | `npx shadcn@latest add accordion` | Expandable sections |
| Collapsible | `npx shadcn@latest add collapsible` | Show/hide content |
| Pagination | `npx shadcn@latest add pagination` | Page navigation |
| Menubar | `npx shadcn@latest add menubar` | Desktop menu bar |
| Tiptap Editor | `npx shadcn@latest add tiptap` | Rich text editor |
| Signature | `npx shadcn@latest add signature` | Canvas-based signature capture |
| Stepper | `npx shadcn@latest add stepper` | Multi-step wizard |
| Table of Contents | `npx shadcn@latest add table-of-contents` | Auto-generated TOC |

## Theming

### CSS Variables

shadcn/ui uses CSS variables for theming. After init, your `globals.css` contains:

```css
@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 0 0% 3.9%;
    --card: 0 0% 100%;
    --card-foreground: 0 0% 3.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 0 0% 3.9%;
    --primary: 0 0% 9%;
    --primary-foreground: 0 0% 98%;
    --secondary: 0 0% 96.1%;
    --secondary-foreground: 0 0% 9%;
    --muted: 0 0% 96.1%;
    --muted-foreground: 0 0% 45.1%;
    --accent: 0 0% 96.1%;
    --accent-foreground: 0 0% 9%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 0 0% 98%;
    --border: 0 0% 89.8%;
    --input: 0 0% 89.8%;
    --ring: 0 0% 3.9%;
    --radius: 0.5rem;
    --chart-1: 12 76% 61%;
    --chart-2: 173 58% 39%;
    --chart-3: 197 37% 24%;
    --chart-4: 43 74% 66%;
    --chart-5: 27 87% 67%;
  }
}
```

### Dark Mode

```tsx
import { useTheme } from "@/hooks/use-theme" // created by shadcn init

export function ThemeToggle() {
  const { setTheme, theme } = useTheme()
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
    >
      {theme === "dark" ? <SunIcon /> : <MoonIcon />}
    </Button>
  )
}
```

### Custom Theme

Modify `globals.css` to change brand colors:

```css
@layer base {
  :root {
    /* Primary brand color */
    --primary: 221 83% 53%; /* Blue */
    --primary-foreground: 0 0% 100%;
  }
  
  .dark {
    --primary: 217 91% 60%;
  }
}
```

## Next.js App Router Integration

### Server vs Client Components

```tsx
// app/page.tsx — Server Component (default)
// Can use: Button, Card, Table, etc. (client-only hooks still needed for interactivity)
import { Button } from "@/components/ui/button"

export default function Page() {
  return (
    <div>
      <h1>Server Component</h1>
      <Button>Click me (hydrates on client)</Button>
    </div>
  )
}
```

```tsx
// components/client-only-form.tsx — Client Component
"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function ClientOnlyForm() {
  const [loading, setLoading] = useState(false)
  
  return (
    <form className="space-y-4">
      <Label htmlFor="name">Name</Label>
      <Input id="name" placeholder="Enter your name" />
      <Button type="submit" disabled={loading}>
        {loading ? "Submitting..." : "Submit"}
      </Button>
    </form>
  )
}
```

### Using with Next.js Server Actions

```tsx
"use server"

export async function createProfile(formData: FormData) {
  const name = formData.get("name") as string
  await db.create({ name })
  revalidatePath("/profiles")
}
```

```tsx
"use client"

import { useFormStatus } from "react-dom"

function SubmitButton() {
  const { pending } = useFormStatus()
  return <Button disabled={pending}>{pending ? "Saving..." : "Save"}</Button>
}
```

## Vite Integration

For Vite/React projects, shadcn/ui works the same way:

```bash
npm create vite@latest my-app -- --template react-ts
cd my-app
npx shadcn@latest init
npx shadcn@latest add button card
```

## Common Patterns

### Composition with Slots

Many shadcn components use `asChild` for composition:

```tsx
import { Button } from "@/components/ui/button"
import Link from "next/link"

<Button asChild>
  <Link href="/dashboard">Go to Dashboard</Link>
</Button>
```

### Controlled vs Uncontrolled

Form components are typically uncontrolled with React Hook Form. For simple controlled state:

```tsx
import { Input } from "@/components/ui/input"
import { useState } from "react"

export function ControlledInput() {
  const [value, setValue] = useState("")
  return <Input value={value} onChange={(e) => setValue(e.target.value)} />
}
```

### Responsive Design

All components work with Tailwind responsive prefixes:

```tsx
<Card className="w-full md:w-[350px] lg:w-[400px]">
  <CardHeader>
    <CardTitle className="text-lg md:text-xl">Responsive Card</CardTitle>
  </CardHeader>
</Card>
```

## Troubleshooting

### Component not found after adding

1. Check the component exists in `components/ui/`
2. Verify `components.json` path configuration
3. Re-run the add command: `npx shadcn@latest add <component>`

### "Cannot find module" errors

Ensure your `tsconfig.json` has proper path aliases matching `components.json`:

```json
{
  "compilerOptions": {
    "paths": {
      "@/*": ["./*"]
    }
  }
}
```

### Dark mode flicker

Wrap your app in `next-themes` `ThemeProvider`:

```tsx
// app/providers.tsx
"use client"

import { ThemeProvider } from "next-themes"

export function Providers({ children }: { children: React.ReactNode }) {
  return <ThemeProvider attribute="class" defaultTheme="system" enableSystem>{children}</ThemeProvider>
}
```

### Radix accessibility warnings in console

This is expected — Radix logs warnings for missing props like `aria-label`. Add accessibility attributes as needed.

## Reference

- Official docs: https://ui.shadcn.com
- Components: https://ui.shadcn.com/docs/components
- CLI: https://ui.shadcn.com/docs/cli
- Figma kit: https://ui.shadcn.com/figma
- GitHub: https://github.com/shadcn-ui/ui
- Radix UI primitives: https://www.radix-ui.com/primitives
