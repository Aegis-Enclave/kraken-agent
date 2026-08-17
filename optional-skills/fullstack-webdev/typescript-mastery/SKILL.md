---
name: typescript-mastery
description: Master TypeScript for professional full-stack development — advanced types, generics, utility types, type guards, conditional types, mapped types, template literal types, declaration merging, module augmentation, and runtime type validation. Use when writing complex type utilities, fixing TypeScript errors, configuring tsconfig, or building type-safe APIs.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [typescript, types, generics, advanced-types, type-safety, zod, type-guard, discriminated-union, template-literal, declaration-files]
    homepage: https://www.typescriptlang.org
    related_skills: [project-scaffolding]
prerequisites:
  typescript: ">=5.0"
  node: ">=18.0.0"
---

# TypeScript Mastery

Master TypeScript for professional full-stack development. This skill covers advanced type system features, patterns for type-safe APIs, and practical techniques used in production codebases.

## When to Use

- Writing complex generic types or utility types
- Fixing TypeScript errors and improving type inference
- Configuring tsconfig.json for different project types
- Building type-safe APIs (REST, GraphQL, tRPC, RPC)
- Creating reusable type utilities for shared packages
- Implementing runtime type validation (Zod, Valibot)
- Writing declaration files (.d.ts) for JavaScript projects
- Working with module augmentation and declaration merging

## TypeScript Configuration

### Modern tsconfig for libraries

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "outDir": "./dist",
    "rootDir": "./src",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true,
    "noPropertyAccessFromIndexSignature": true,
    "forceConsistentCasingInFileNames": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### tsconfig for Next.js

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

## Utility Types Deep Dive

### Pick, Omit, Partial, Required

```typescript
interface User {
  id: string
  name: string
  email: string
  createdAt: Date
  role: 'admin' | 'user' | 'guest'
}

// Pick specific fields
type UserPreview = Pick<User, 'id' | 'name'>

// Omit fields you don't need
type CreateUserInput = Omit<User, 'id' | 'createdAt'>
// { name: string; email: string; role: 'admin' | 'user' | 'guest' }

// Make all properties optional (for PATCH operations)
type UpdateUserInput = Partial<User>

// Make all properties required
type CompleteUser = Required<User>

// All properties become readonly
type FrozenUser = Readonly<User>
```

### Record, Map, Infer

```typescript
// Record for key-value maps
type UserRoles = Record<string, 'admin' | 'editor' | 'viewer'>
const roles: UserRoles = {
  alice: 'admin',
  bob: 'editor',
}

// Extract keys as a union type
type UserRoleKeys = keyof UserRoles
// string — but you can constrain it:
type ConstrainedRoles = Record<'admin' | 'editor' | 'viewer', boolean>

// Infer type from a return type
async function fetchUser(): Promise<User> {
  return { id: '1', name: 'Alice', email: 'a@b.com', createdAt: new Date(), role: 'user' }
}
type FetchedUser = Awaited<ReturnType<typeof fetchUser>>
// User

// Infer parameter types
function createUser(data: { name: string; email: string }) { /* ... */ }
type CreateUserParams = Parameters<typeof createUser>[0]
// { name: string; email: string }
```

## Generics Advanced Patterns

### Constrained generics

```typescript
// Ensure the type has an 'id' field
function findById<T extends { id: string }>(items: T[], id: string): T | undefined {
  return items.find(item => item.id === id)
}

// Ensure the type can be serialized
function cache<T extends object>(key: string, value: T): void {
  localStorage.setItem(key, JSON.stringify(value))
}

// Generic with multiple constraints
function merge<T extends object, U extends object>(target: T, source: U): T & U {
  return { ...target, ...source }
}
```

### Generic inference with `infer`

```typescript
// Extract the element type from an array
type ArrayElement<T> = T extends readonly (infer E)[] ? E : never
type Strings = ArrayElement<string[]>  // string

// Extract return type
type ReturnTypeOf<T> = T extends (...args: any[]) => infer R ? R : never
type Result = ReturnTypeOf<() => Promise<User>>  // Promise<User>

// Extract first argument
type FirstArg<T> = T extends (first: infer A, ...rest: any[]) => any ? A : never
type CallbackArg = FirstArg<(data: { id: string }) => void>  // { id: string }

// Extract promise resolved type
type Resolved<T> = T extends Promise<infer U> ? U : T
type Data = Resolved<Promise<string[]>>  // string[]
```

### Conditional types with generics

```typescript
// Type based on property presence
type ApiResponse<T> = T extends { data: infer D } 
  ? { success: true; data: D }
  : { success: false; error: string }

// Distributive conditional types
type NonNullable<T> = T extends null | undefined ? never : T
type Strings = NonNullable<string | null | undefined | number>  // string | number

// Filter union types
type FilterFlags<T, Flag> = {
  [K in keyof T]: T[K] extends Flag ? K : never
}
type StringKeys<T> = FilterFlags<T, string>[keyof T]
type UserStringProps = StringKeys<User>  // 'name' | 'email'

// Exclude / Extract
type Admin = Extract<User['role'], 'admin'>  // 'admin'
type NonAdmin = Exclude<User['role'], 'admin'>  // 'user' | 'guest'
```

## Type Guards & Narrowing

### User-defined type guards

```typescript
interface Cat {
  meow(): void
}
interface Dog {
  bark(): void
}

function isCat(animal: Cat | Dog): animal is Cat {
  return (animal as Cat).meow !== undefined
}

function makeSound(animal: Cat | Dog) {
  if (isCat(animal)) {
    animal.meow()
  } else {
    animal.bark()
  }
}
```

### Using `in` operator

```typescript
type ApiError = { error: string; code: number }
type ApiSuccess<T> = { data: T }

function isSuccess<T>(response: ApiError | ApiSuccess<T>): response is ApiSuccess<T> {
  return 'data' in response
}

function handleResponse<T>(response: ApiError | ApiSuccess<T>) {
  if (isSuccess(response)) {
    console.log(response.data)  // TypeScript knows it's ApiSuccess<T>
  } else {
    console.error(response.error)
  }
}
```

### Discriminated unions

```typescript
type LoadingState = 
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: User[] }
  | { status: 'error'; error: string }

function render(state: LoadingState) {
  switch (state.status) {
    case 'idle':
      return 'Ready'
    case 'loading':
      return 'Loading...'
    case 'success':
      return state.data.length  // data is available here
    case 'error':
      return state.error  // error is available here
  }
}
```

## Template Literal Types

### String manipulation

```typescript
// Create event types from names
type EventName = 'click' | 'focus' | 'blur'
type CustomEvent = `on${Capitalize<EventName>}`  // 'onClick' | 'onFocus' | 'onBlur'

// Extract path parameters
type Route = '/users/:id/posts/:postId'
type Params<T extends string> = 
  T extends `${string}:${infer Param}/${infer Rest}` 
    ? Param | Params<`/${Rest}`>
    : T extends `${string}:${infer Param}` 
      ? Param 
      : never

type RouteParams = Params<Route>  // 'id' | 'postId'

// Build API method types
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
type ApiPath = '/users' | '/posts' | '/comments'
type Endpoint = `${HttpMethod} ${ApiPath}`

// JSON key transformations
type SnakeToCamel<S extends string> = 
  S extends `${infer T}_${infer U}` 
    ? `${T}${Capitalize<SnakeToCamel<U>>}` 
    : S

type CamelKeys<T> = {
  [K in keyof T as SnakeToCamel<string & K>]: T[K]
}

interface SnakeCaseUser {
  user_id: string
  created_at: string
}
type CamelCaseUser = CamelKeys<SnakeCaseUser>
// { userId: string; createdAt: string }
```

## Mapped Types

### Transform object types

```typescript
// Make all values optional or required
type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K]
}

type DeepReadonly<T> = {
  readonly [K in keyof T]: T[K] extends object ? DeepReadonly<T[K]> : T[K]
}

// Rename keys
type Rename<T, R extends { [K in keyof R]: string }> = {
  [K in keyof T as K extends keyof R ? R[K] : K]: T[K]
}

// Add properties
type WithMeta<T> = T & {
  createdAt: Date
  updatedAt: Date
  id: string
}

// Filter by value type
type PickByValue<T, V> = {
  [K in keyof T as T[K] extends V ? K : never]: T[K]
}

type StringProps = PickByValue<User, string>  // Only string properties

// Conditional mapped types
type Getters<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K]
}
type UserGetters = Getters<User>
// { getId: () => string; getName: () => string; ... }
```

## Zod for Runtime Validation

### Define schemas with Zod

```bash
pnpm add zod
```

```typescript
import { z } from 'zod'

// Basic schemas
const UserSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(2).max(100),
  email: z.string().email(),
  age: z.number().int().positive().optional(),
  role: z.enum(['admin', 'user', 'guest']),
  createdAt: z.coerce.date(),  // String to Date
})

// Infer TypeScript type from schema
type User = z.infer<typeof UserSchema>

// Create with defaults
const UserWithDefaults = UserSchema.merge(z.object({
  id: z.string().uuid().default(crypto.randomUUID),
  createdAt: z.coerce.date().default(() => new Date()),
}))

// Transform data
const UserInput = z.object({
  name: z.string(),
  email: z.string().email(),
  password: z.string().min(8),
}).transform(({ password, ...rest }) => ({
  ...rest,
  passwordHash: await hash(password),  // hashSync for sync
}))

// Parse with error handling
const result = UserSchema.safeParse(data)

if (!result.success) {
  console.log(result.error.flatten())
  // { fieldErrors: { email: ['Invalid email'] } }
} else {
  const user = result.data  // User
}
```

### Schema composition

```typescript
// Partial for updates
const UpdateUserSchema = UserSchema.partial()

// Pick/Omit
const PublicUserSchema = UserSchema.omit({ passwordHash: true })

// Extend
const AdminSchema = UserSchema.extend({
  permissions: z.array(z.string())
})

// Merge
const TimestampedUser = UserSchema.merge(z.object({
  createdAt: z.date(),
  updatedAt: z.date(),
}))

// Array validation
const UsersArraySchema = z.array(UserSchema)

// Union types
const ErrorOrUser = z.union([
  z.object({ error: z.string() }),
  UserSchema
])
```

### Validate API inputs

```typescript
// Express middleware
import { z } from 'zod'

const createUserSchema = z.object({
  name: z.string().min(2),
  email: z.string().email(),
  password: z.string().min(8)
})

app.post('/users', async (req, res) => {
  const result = createUserSchema.safeParse(req.body)
  
  if (!result.success) {
    return res.status(400).json({
      error: 'Validation failed',
      details: result.error.flatten()
    })
  }
  
  const { name, email, password } = result.data
  // TypeScript knows these are valid strings
})

// With React Hook Form
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'

const { register, handleSubmit } = useForm({
  resolver: zodResolver(createUserSchema)
})
```

## Declaration Merging & Module Augmentation

### Extend existing types

```typescript
// Extend the Window interface
declare global {
  interface Window {
    analytics: {
      track(event: string, data?: Record<string, unknown>): void
    }
  }
}

// Extend Express Request
declare global {
  namespace Express {
    interface Request {
      user?: { id: string; role: string }
      requestId: string
    }
  }
}

// Augment a library's types
import { Mongoose } from 'mongoose'

declare module 'mongoose' {
  interface Document {
    softDelete(): Promise<this>
    restoredAt?: Date
  }
}
```

### Declaration files for JavaScript

```typescript
// types/user.d.ts
export interface User {
  id: string
  name: string
  email: string
}

export interface UserRepository {
  findById(id: string): Promise<User | null>
  findAll(): Promise<User[]>
  create(data: Omit<User, 'id'>): Promise<User>
  update(id: string, data: Partial<User>): Promise<User | null>
  delete(id: string): Promise<boolean>
}

// types/utilities.d.ts
export declare function hashPassword(password: string): Promise<string>
export declare function comparePasswords(password: string, hash: string): Promise<boolean>
```

## Advanced Type Patterns

### Recursive types

```typescript
// JSON type
type JSONPrimitive = string | number | boolean | null
type JSONValue = JSONPrimitive | JSONValue[] | { [key: string]: JSONValue }
type JSONObject = { [key: string]: JSONValue }

// Tree structure
interface TreeNode<T> {
  value: T
  children: TreeNode<T>[]
}

// Deep keys for nested objects
type DeepKeys<T, Prefix extends string = ''> = T extends object
  ? {
      [K in keyof T & string]: 
        | `${Prefix}${K}`
        | (T[K] extends object ? DeepKeys<T[K], `${Prefix}${K}.`> : never)
    }[keyof T & string]
  : never

type UserDeepKeys = DeepKeys<User>  // 'id' | 'name' | 'email' | 'role' | 'createdAt'
```

### Variadic tuple types

```typescript
// Concatenate arrays
type Concat<T extends unknown[], U extends unknown[]> = [...T, ...U]
type Flatten<T extends any[][]> = T[number]

// Merge two functions
type Compose<F extends (...args: any[]) => any, G extends (...args: any[]) => any> = 
  (...args: Parameters<G>) => ReturnType<F> extends Parameters<G>[number] 
    ? ReturnType<G> 
    : ReturnType<F>

// Prefix types
type Prefixed<T extends string[], P extends string> = {
  [K in T[number]]: `${P}${K}`
}
```

### Brand types for type safety

```typescript
// Nominal typing for primitives
type Brand<T, B extends string> = T & { readonly _brand: B }

type UserId = Brand<string, 'UserId'>
type PostId = Brand<string, 'PostId'>
type Email = Brand<string, 'Email'>

function createUserId(id: string): UserId {
  // Validate format
  if (!isValidUUID(id)) throw new Error('Invalid user ID')
  return id as UserId
}

function getUser(id: UserId) { /* ... */ }

const userId = createUserId('123')
getUser(userId)  // OK
getUser('123' as string)  // Error: string is not UserId
```

## Best Practices

1. **Enable strict mode** — catches more bugs at compile time
2. **Prefer `type` over `interface`** for consistency, use `interface` for extension points
3. **Use `unknown` over `any`** — forces runtime type checking
4. **Leverage discriminated unions** — for state machines and API responses
5. **Use `z.infer<>`** — single source of truth for runtime + compile types
6. **Avoid `as`** — use type guards instead when possible
7. **Export utility types** — from shared packages for reuse
8. **Document complex types** — add comments for intricate generics

## Common TypeScript Errors & Fixes

### 'X' is possibly undefined

```typescript
// Problem
const name = user?.profile?.name  // string | undefined

// Solutions
// 1. Optional chaining with nullish coalescing
const name = user?.profile?.name ?? 'Anonymous'

// 2. Enable noUncheckedIndexedAccess
// tsconfig: "noUncheckedIndexedAccess": true

// 3. Type guard
if (user?.profile?.name) {
  console.log(user.profile.name.toUpperCase())
}
```

### Type 'X' is not assignable to type 'Y'

```typescript
// Problem: index signature mismatch
interface Config { [key: string]: string }
const config: Config = { port: '3000' }

// Fix 1: Use extends constraint
interface TypedConfig<T extends Record<string, unknown>> {
  [K in keyof T]: T[K]
}

// Fix 2: Use satisfies
const config = { port: '3000' } satisfies Record<string, string>
```

### Generic type instantiation is excessively deep

```typescript
// Problem: circular or nested generics
type DeepPick<T, K extends keyof T> = 
  K extends keyof T 
    ? { [P in K]: T[P] } 
    : never

// Fix: Add recursion depth limit
type DeepPick<T, K extends keyof T, D extends number = 5> = 
  D extends 0 ? {} :
  K extends keyof T 
    ? { [P in K]: T[P] extends object ? DeepPick<T[P], keyof T[P], D[-1]> : T[P] } 
    : {}
```

## Guardrails

- Never use `// @ts-ignore` without a comment explaining why
- Avoid `any` — use `unknown` and narrow appropriately
- Don't over-engineer types — prefer simple types until complexity is needed
- Test your types — use `type-fest` and `tsd` for validation
- Keep types in sync with runtime validation using Zod
