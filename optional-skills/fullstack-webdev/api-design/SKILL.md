---
name: api-design
description: "REST and GraphQL API design patterns for production web apps — naming, versioning, pagination, error shapes, auth, OpenAPI specs."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [api, rest, graphql, tRPC, openapi, backend, http, versioning]
    related_skills: [nodejs-backend-patterns, fastapi-pro, vercel-deployment]
---

# API Design

## Core Principles

1. **Consistency** — same pattern everywhere, users can predict behavior
2. **Clarity** — naming reflects resources, not actions
3. **Pragmatism** — REST is a style guide, not a religion
4. **Documentation** — OpenAPI spec from day one

---

## REST Conventions

### Resource Naming

Use **plural nouns**, lowercase, hyphenated:

```
GET    /users              # list
GET    /users/:id          # get one
POST   /users              # create
PATCH  /users/:id          # partial update
PUT    /users/:id          # full replace (rare)
DELETE /users/:id          # delete
POST   /users/:id/deactivate  # actions on resources
```

Avoid verbs in URLs — the HTTP method IS the verb.

### HTTP Status Codes

| Code | Meaning | When to use |
|------|---------|-------------|
| 200  | OK | Successful GET, PATCH |
| 201  | Created | Successful POST that creates |
| 204  | No Content | Successful DELETE |
| 400  | Bad Request | Validation error, malformed input |
| 401  | Unauthorized | Missing or invalid auth |
| 403  | Forbidden | Authenticated but no permission |
| 404  | Not Found | Resource doesn't exist |
| 409  | Conflict | Duplicate, version conflict |
| 422  | Unprocessable Entity | Validation error (more specific than 400) |
| 429  | Too Many Requests | Rate limited |
| 500  | Internal Server Error | Unhandled error |

### Error Response Shape

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      { "field": "email", "message": "Must be a valid email address" },
      { "field": "password", "message": "Must be at least 8 characters" }
    ],
    "request_id": "req_01HXYZ"
  }
}
```

Always include a `request_id` for tracing.

### Pagination

**Cursor-based (preferred)** — stable under concurrent inserts:

```
GET /posts?limit=20&cursor=eyJpZCI6MTIzfQ
```

Response:
```json
{
  "data": [...],
  "pagination": {
    "has_next": true,
    "next_cursor": "eyJpZCI6MTQzfQ",
    "total": null
  }
}
```

**Offset/limit** — only for admin UIs or where total count matters:
```
GET /posts?offset=0&limit=20
```

**Never** paginate by page number — breaks on concurrent inserts.

### Filtering & Sorting

```
GET /posts?status=published&author_id=123&sort=created_at&order=desc
GET /posts?tags[]=react&tags[]=typescript
```

---

## API Versioning

### URL Path (recommended)

```
/v1/users
/v2/users
```

Pros: clear, easy to route, cacheable
Cons: URL changes

### Header Versioning

```
Accept: application/vnd.api+json; version=2
```

More complex. Use only when you can't change URLs.

**Never** mix versioning strategies. Pick one and stick to it.

---

## GraphQL Design

### Schema-First

Define the schema before writing resolvers:

```graphql
type Post {
  id: ID!
  title: String!
  body: String!
  author: User!
  tags: [Tag!]!
  createdAt: DateTime!
  publishedAt: DateTime
}

type Query {
  posts(first: Int, after: String, tag: String): PostConnection!
  post(id: ID!): Post
}

type Mutation {
  createPost(input: CreatePostInput!): Post!
  publishPost(id: ID!): Post!
}

type PostConnection {
  edges: [PostEdge!]!
  pageInfo: PageInfo!
}
```

### N+1 Problem

Always use DataLoader to batch and cache:

```typescript
const userLoader = new DataLoader<string, User>(async (ids) => {
  const users = await db.users.findMany({ where: { id: { in: ids } } });
  return ids.map(id => users.find(u => u.id === id));
});

const resolvers = {
  Post: {
    author: (post) => userLoader.load(post.authorId)
  }
};
```

### Mutations

Use input types and return the modified entity:

```graphql
mutation CreatePost($input: CreatePostInput!) {
  createPost(input: $input) {
    id
    title
    publishedAt
  }
}
```

### Error Handling

```graphql
type Mutation {
  createPost(input: CreatePostInput!): PostPayload!
}

union PostPayload = Post | ValidationError | AuthorizationError
```

Or use extensions on errors:

```json
{
  "errors": [{
    "message": "Not authorized",
    "extensions": { "code": "FORBIDDEN" }
  }]
}
```

---

## tRPC Patterns

For TypeScript apps, tRPC eliminates the need for an API spec:

```typescript
const appRouter = router({
  post: router({
    list: publicProcedure
      .input(z.object({ limit: z.number().min(1).max(100).default(20), cursor: z.string().nullish() }))
      .query(async ({ input }) => { /* ... */ }),

    byId: publicProcedure
      .input(z.object({ id: z.string() }))
      .query(async ({ input }) => { /* ... */ }),

    create: protectedProcedure
      .input(z.object({ title: z.string().min(1), body: z.string() }))
      .mutation(async ({ ctx, input }) => { /* ... */ }),
  }),
});
```

Always use Zod schemas for input validation. Return the full entity on mutations.

---

## Request Validation

Validate at the boundary — never trust client data:

```typescript
// Zod
const CreateUserSchema = z.object({
  email: z.string().email(),
  name: z.string().min(1).max(100),
  role: z.enum(['admin', 'user']).default('user'),
});

app.post('/users', async (req, res) => {
  const result = CreateUserSchema.safeParse(req.body);
  if (!result.success) {
    return res.status(422).json({ error: result.error.flatten() });
  }
  // proceed with result.data
});
```

---

## Rate Limiting

Return headers so clients can back off:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699999999
Retry-After: 30
```

Use a sliding window in Redis for per-user limits.

---

## OpenAPI 3.1 Spec

Generate from code (tsoa, swagger-jsdoc, or zod-to-openapi):

```yaml
openapi: 3.1.0
info:
  title: My API
  version: 1.0.0
paths:
  /users:
    get:
      summary: List users
      parameters:
        - name: limit
          in: query
          schema: { type: integer, default: 20, maximum: 100 }
        - name: cursor
          in: query
          schema: { type: string }
      responses:
        '200':
          description: Paginated list of users
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserList'
components:
  schemas:
    UserList:
      type: object
      properties:
        data:
          type: array
          items: { $ref: '#/components/schemas/User' }
        pagination:
          $ref: '#/components/schemas/Pagination'
```

---

## Idempotency

For POST requests, support idempotency keys:

```
POST /payments
Idempotency-Key: 01HXYZ

# Same response on retry
```

Store idempotency keys with response in Redis (TTL: 24h).
